#![feature(proc_macro_hygiene, decl_macro)]

use api::Context;
use rocket::{catch, catchers, fairing::AdHoc, get, post, routes, Rocket, State};
use slog_scope::debug;
use std::sync::{Arc, Mutex};
use thiserror::Error;

mod api;
pub mod config;
mod global;
mod password;
pub mod root_auth;
pub mod secret_key;
pub mod test_util;

pub use api::TokenMgr;
use api::TokenMgrError;
pub use config::FrontendParams;
pub use root_auth::LocalAuthServer;

type DbPool = Arc<dyn db::DbConn>;

#[catch(400)]
fn catch_bad_request() -> &'static str {
    r#"
Your request is incorrect.
Possible reasons:
- Query body is missing or is not valid JSON
- X-Jjs-Auth header is not valid access token
    "#
}

#[get("/")]
fn route_ping() -> &'static str {
    "JJS frontend: pong"
}

#[get("/graphiql")]
fn route_graphiql() -> rocket::response::content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

struct JuniperResponseDebug(juniper_rocket::GraphQLResponse);

impl std::fmt::Debug for JuniperResponseDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let resp = &self.0;
        std::fmt::Debug::fmt(&resp.1, f)
    }
}

type BadRequestResponder = rocket::response::status::BadRequest<String>;

fn execute_request(
    req: juniper_rocket::GraphQLRequest,
    schema: &api::Schema,
    ctx: &Result<api::Context, TokenMgrError>,
) -> Result<juniper_rocket::GraphQLResponse, BadRequestResponder> {
    match ctx {
        Ok(ctx) => {
            let res = req.execute(schema, ctx);

            let res = JuniperResponseDebug(res);

            debug!("API request"; "request" => ?req, "response" => ?res);

            Ok(res.0)
        }
        Err(err) => {
            let error_message = if cfg!(debug_assertions) {
                format!("bad request: {}", err)
            } else {
                r#"
Your request is incorrect.
Possible reasons:
- Query body is missing or is not valid JSON
- X-Jjs-Auth header is not valid access token
    "#
                .to_string()
            };
            Err(rocket::response::status::BadRequest(Some(error_message)))
        }
    }
}

#[get("/graphql?<request>")]
fn route_get_graphql(
    ctx: Result<api::Context, TokenMgrError>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<api::Schema>,
) -> Result<juniper_rocket::GraphQLResponse, BadRequestResponder> {
    execute_request(request, &*schema, &ctx)
}

#[post("/graphql", data = "<request>")]
fn route_post_graphql(
    ctx: Result<api::Context, TokenMgrError>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<api::Schema>,
) -> Result<juniper_rocket::GraphQLResponse, BadRequestResponder> {
    execute_request(request, &*schema, &ctx)
}

#[derive(Clone)]
struct GqlApiSchema(String);

#[get("/graphql/schema")]
fn route_get_graphql_schema(schema: State<GqlApiSchema>) -> String {
    schema.clone().0
}

#[post("/internal/lsu-webhook?<token>", data = "<lsu>")]
fn route_lsu_webhook(
    global_state: State<Arc<Mutex<global::GlobalState>>>,
    lsu: rocket_contrib::json::Json<invoker_api::LiveStatusUpdate>,
    token: String,
) {
    global_state
        .lock()
        .unwrap()
        .live_status_updates
        .webhook_handler(lsu.into_inner(), token);
}

#[derive(Error, Debug)]
pub enum ApiServerCreateError {
    #[error("failed to initialize Rocket: {0}")]
    Rocket(#[from] rocket::config::ConfigError),
    #[error("failed to intropspect api: {0}")]
    IntrospectGql(String),
}

pub struct ApiServer {}

impl ApiServer {
    pub fn create_embedded() -> Rocket {
        let db_conn: Arc<dyn db::DbConn> = db::connect::connect_memory().unwrap().into();
        let builder = entity::loader::LoaderBuilder::new();
        let secret: Arc<[u8]> = config::derive_key_512("EMBEDDED_FRONTEND_INSTANCE")
            .into_boxed_slice()
            .into();
        let token_mgr = crate::api::TokenMgr::new(db_conn.clone(), secret);
        let frontend_config = config::FrontendParams {
            cfg: config::FrontendConfig {
                port: 0,
                addr: Some("127.0.0.1".to_string()),
                host: "127.0.0.1".to_string(),
                unix_socket_path: "".to_string(),
                env: config::Env::Dev,
                tls: None,
            },
            token_mgr,
            db_conn: db_conn.clone(),
        };

        Self::create(
            Arc::new(frontend_config),
            builder.into_inner(),
            db_conn,
            problem_loader::Loader::empty(),
            std::path::Path::new("/tmp/jjs"),
        )
        .expect("failed to create embedded instance")
    }

    pub fn get_schema() -> String {
        let rock = Self::create_embedded();
        rock.state::<GqlApiSchema>().unwrap().0.clone()
    }

    pub fn create(
        frontend_params: Arc<config::FrontendParams>,
        entity_loader: entity::Loader,
        pool: DbPool,
        problem_loader: problem_loader::Loader,
        data_dir: &std::path::Path,
    ) -> Result<Rocket, ApiServerCreateError> {
        let rocket_cfg_env = match frontend_params.cfg.env {
            config::Env::Prod => rocket::config::Environment::Production,
            config::Env::Dev => rocket::config::Environment::Development,
        };
        let mut rocket_config = rocket::Config::new(rocket_cfg_env);

        rocket_config.set_address(frontend_params.cfg.host.clone())?;
        rocket_config.set_port(frontend_params.cfg.port);
        rocket_config.set_log_level(match frontend_params.cfg.env {
            config::Env::Dev => rocket::config::LoggingLevel::Normal,
            config::Env::Prod => rocket::config::LoggingLevel::Critical,
        });
        rocket_config
            .set_secret_key(base64::encode(frontend_params.token_mgr.secret_key()))
            .unwrap();
        if let Some(tls) = &frontend_params.cfg.tls {
            rocket_config.set_tls(&tls.cert_path, &tls.key_path)?;
        }

        let graphql_context_factory = api::ContextFactory {
            pool: Arc::clone(&pool),
            cfg: Arc::new(entity_loader),
            fr_cfg: Arc::clone(&frontend_params),
            problem_loader: Arc::new(problem_loader),
            data_dir: data_dir.into(),
        };

        let graphql_schema = api::Schema::new(api::Query, api::Mutation);

        let (intro_data, intro_errs) = juniper::introspect(
            &graphql_schema,
            &Context(Arc::new(
                graphql_context_factory.create_context_data_unrestricted(),
            )),
            juniper::IntrospectionFormat::default(),
        )
        // TODO: this is hack
        .map_err(|err| ApiServerCreateError::IntrospectGql(format!("{:?}", err)))?;
        assert!(intro_errs.is_empty());

        let introspection_json = serde_json::to_string(&intro_data).unwrap();

        let cfg1 = Arc::clone(&frontend_params);

        Ok(rocket::custom(rocket_config)
            .manage(graphql_context_factory)
            .manage(graphql_schema)
            .manage(GqlApiSchema(introspection_json))
            .manage(Arc::new(Mutex::new(global::GlobalState::new())))
            .manage(frontend_params)
            .attach(AdHoc::on_attach("ProvideSecretKey", move |rocket| {
                Ok(rocket.manage(secret_key::SecretKey(cfg1.token_mgr.secret_key().into())))
            }))
            .mount(
                "/",
                routes![
                    route_get_graphql_schema,
                    route_graphiql,
                    route_get_graphql,
                    route_post_graphql,
                    route_ping,
                    route_lsu_webhook
                ],
            )
            .register(catchers![catch_bad_request]))
    }
}
