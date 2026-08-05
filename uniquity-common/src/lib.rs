//! Shared helpers for Uniquity Ventures plugins.

pub mod decimal;
pub mod schema;
pub mod typst;

use lariv_rs::plugins::users::state::AuthContext;

/// Whether the user has superuser access (all Uniquity apps require this).
pub fn is_superuser(auth: &AuthContext) -> bool {
    auth.user.is_superuser
}

/// Deny write access for non-superuser users.
pub fn require_superuser(auth: &AuthContext) -> bool {
    is_superuser(auth)
}
