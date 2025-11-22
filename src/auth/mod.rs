pub mod api_key;
pub mod auth_service;
pub mod guards;
pub mod middleware;
pub mod oauth;
pub mod session;

use crate::models::auth::Role;
use crate::models::member::Member;

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user: Option<Member>,
}

impl AuthContext {
    pub fn new(user: Option<Member>) -> Self {
        Self { user }
    }

    pub fn user(&self) -> Option<&Member> {
        self.user.as_ref()
    }

    pub fn role(&self) -> Option<Role> {
        self.user.as_ref().map(|u| u.role)
    }

    pub fn has_role(&self, role: Role) -> bool {
        self.role() == Some(role)
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(Role::Admin)
    }

    pub fn is_bot(&self) -> bool {
        self.has_role(Role::Bot)
    }
}
