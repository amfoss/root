use crate::auth::AuthContext;
use async_graphql::{Context, Error, Guard, Result};

pub struct AuthGuard;

impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let auth = ctx.data::<AuthContext>().map_err(|_| {
            Error::new("Authentication context not found. This is an internal server error.")
        })?;

        if auth.is_authenticated() {
            Ok(())
        } else {
            Err(Error::new(
                "Authentication required to access this resource.",
            ))
        }
    }
}

pub struct AdminGuard;

impl Guard for AdminGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let auth = ctx.data::<AuthContext>().map_err(|_| {
            Error::new("Authentication context not found. This is an internal server error.")
        })?;

        if auth.is_admin() {
            Ok(())
        } else {
            Err(Error::new("Admin privileges required for this operation"))
        }
    }
}

pub struct AdminOrBotGuard;

impl Guard for AdminOrBotGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let auth = ctx.data::<AuthContext>().map_err(|_| {
            Error::new("Authentication context not found. This is an internal server error.")
        })?;

        if auth.is_bot() || auth.is_admin() {
            Ok(())
        } else {
            Err(Error::new(
                "Admin or Bot privileges required for this operation",
            ))
        }
    }
}
