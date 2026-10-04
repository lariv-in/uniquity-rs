//! Shared helpers for Uniquity Ventures plugins.

pub mod decimal;
pub mod schema;
pub mod typst;

use lariv_rs::plugins::users::roles::Superuser;
use lariv_rs::plugins::users::state::AuthContext;

/// Whether the user has the superuser role.
pub fn is_superuser(auth: &AuthContext) -> bool {
    Superuser::matches(&auth.role)
}

/// Whether the user has the superuser role.
pub fn require_superuser(auth: &AuthContext) -> bool {
    is_superuser(auth)
}
