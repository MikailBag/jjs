use super::prelude::*;

pub(super) fn simple(
    ctx: &Context,
    login: String,
    password: String,
) -> ApiResult<schema::SessionToken> {
    let mut success = false;
    let mut reject_reason = "";
    if let Some(user) = ctx.db.user_try_load_by_login(&login).internal(ctx)? {
        success = crate::password::check_password_hash(&password, &user.password_hash);
        if !success {
            reject_reason = "IncorrectPassword";
        }
    } else {
        reject_reason = "UnknownUser";
    }
    if success {
        let token = ctx.token_mgr.create_token(&login).internal(ctx)?;
        let buf = token.serialize(&ctx.secret_key);
        let sess = schema::SessionToken {
            data: buf,
            raw_data: None, //TODO
        };
        Ok(sess)
    } else {
        let mut ext = ErrorExtension::new();
        ext.set_error_code(reject_reason);
        let err = ApiError {
            visible: true,
            extension: ext,
            source: None,
            ctx: ctx.clone(),
        };
        Err(err)
    }
}
