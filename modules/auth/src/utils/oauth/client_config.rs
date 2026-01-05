use compact_str::CompactString;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct OAuthProviderClientConfig {
    /// The public identifier for your application, provided by the OAuth provider
    pub client_id: CompactString,
    /// The secret key for your application (keep this secure and never expose it publicly)
    pub client_secret: CompactString,
}

impl std::fmt::Debug for OAuthProviderClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthProviderClientConfig")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .finish()
    }
}
