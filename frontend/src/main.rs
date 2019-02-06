#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::{http::Status, State};
use rocket_contrib::json::Json;
//use rocket::response::Responder;
#[get("/ping")]
fn route_ping() -> &'static str {
    "JJS frontend"
}

#[derive(Debug)]
enum FrontendError {
    Internal,
}

impl<'r> rocket::response::Responder<'r> for FrontendError {
    fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'r> {
        let res = match self {
            FrontendError::Internal => Status::InternalServerError,
        };
        Err(res)
        //res.respond_to(request)
    }
}

type FrontendResult<T> = Result<T, FrontendError>;

type DbPool =r2d2::Pool<r2d2_postgres::PostgresConnectionManager>;

#[post("/auth/anonymous")]
fn route_auth_anonymous() -> Result<Json<frontend_api::AuthToken>, FrontendError> {
    Ok(Json(frontend_api::AuthToken {
        buf: "".as_bytes().to_vec(),
    }))
    //FrontendError::Internal
}

//
#[post("/submission/send", data = "<data>")]
fn route_submissions_send(
    data: Json<frontend_api::SubmitDeclaration>,
    db: State<DbPool>,
) -> Result<Json<Result<frontend_api::SubmissionId, frontend_api::SubmitError>>, FrontendError> {
    use std::ops::Deref;
    let conn = db.get().expect("couldn't connect to DB");
    let db = db::Db::new(conn.deref());
    let res = db.submissions.create_submission(&data.toolchain);
    let res = Ok(res.id);
    Ok(Json(res))
}
/*impl frontend_api::JjsServiceSyncHandler for Api {
    fn handle_anon(&self) -> thrift::Result<frontend_api::AuthToken> {
        let s = "_".to_string();
        let buf = s.into_bytes();
        Ok(frontend_api::AuthToken { buf })
    }

    fn handle_simple(
        &self,
        params: frontend_api::SimpleAuthParams,
    ) -> thrift::Result<frontend_api::AuthToken> {
        let s = format!("${}", &params.login);
        let buf = s.into_bytes();
        Ok(frontend_api::AuthToken { buf })
    }

    fn handle_drop(&self, _token: frontend_api::AuthToken) -> thrift::Result<()> {
        //TODO implement
        Ok(())
    }

    fn handle_submit(
        &self,
        params: frontend_api::SubmitDeclaration,
    ) -> thrift::Result<frontend_api::SubmissionId> {
        let ctx = self.ctx_provider.provide();
        let s8n = ctx.db.submissions.create_submission(&params.toolchain);
        Ok(s8n.id as i64)
    }

    fn handle_ping(&self, buf: String) -> thrift::Result<String> {
        Ok(buf)
    }
}
*/
fn main() {
    dotenv::dotenv().ok();
    let port = 1779;
    let listen_address = format!("127.0.0.1:{}", port);
    let postgress_url =
        std::env::var("POSTGRES_URL").expect("'POSTGRES_URL' environment variable is not set");
    let pg_conn_manager =
        r2d2_postgres::PostgresConnectionManager::new(postgress_url, r2d2_postgres::TlsMode::None)
            .expect("coudln't initialize DB connection pool");
    let pool = r2d2::Pool::new(pg_conn_manager).expect("coudln't initialize DB connection pool");
    println!("JJS api frontend is listening on {}", &listen_address);
    rocket::ignite()
        .manage(pool)
        .mount(
            "/",
            routes![route_ping, route_auth_anonymous, route_submissions_send],
        )
        .launch();
}
