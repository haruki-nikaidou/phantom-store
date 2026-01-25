pub mod client_config;
pub mod providers;

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Common structure for OAuth user information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthUserInfo {
    /// Unique identifier for the user from the provider
    pub id: CompactString,
    /// User's email address (if available)
    pub email: Option<CompactString>,
    /// Whether the email is verified (if available)
    pub email_verified: Option<bool>,
    /// User's display name or full name (if available)
    pub name: Option<CompactString>,
    /// URL to the user's profile picture (if available)
    pub picture: Option<CompactString>,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthUserInfoError {
    #[error("Failed to fetch user info: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("User info URL not provided")]
    NoUserInfoUrl,

    #[error("Failed to parse user info: {0}")]
    ParseError(String),
}
