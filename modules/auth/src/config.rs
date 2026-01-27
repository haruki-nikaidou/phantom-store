use crate::utils::oauth::client_config::OAuthProviderClientConfig;
use crate::utils::oauth::providers::OAuthProviderName;
use compact_str::CompactString;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EmailDomainConfig {
    pub enable_whitelist: bool,
    pub whitelisted_domains: Box<[CompactString]>,
    pub enable_blacklist: bool,
    pub blacklisted_domains: Box<[CompactString]>,
}

impl EmailDomainConfig {
    /// Check if the email address is allowed to use.
    pub fn check_addr(&self, addr: impl AsRef<str>) -> bool {
        let addr = addr.as_ref();
        let Ok(address): Result<lettre::Address, _> = addr.parse() else {
            return false;
        };
        let domain = address.domain();
        if self.enable_whitelist
            && !self
                .whitelisted_domains
                .contains(&CompactString::new(domain))
        {
            return false;
        }
        if self.enable_blacklist
            && self
                .blacklisted_domains
                .contains(&CompactString::new(domain))
        {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailOtpConfig {
    pub expire_after: time::Duration,
    pub delete_before: time::Duration,
    pub resend_interval: time::Duration,
}

impl Default for EmailOtpConfig {
    fn default() -> Self {
        Self {
            expire_after: time::Duration::minutes(10),
            delete_before: time::Duration::hours(2),
            resend_interval: time::Duration::minutes(1),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EmailProviderConfig {
    pub domain: EmailDomainConfig,
    pub otp: EmailOtpConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionConfig {
    pub session_ttl: time::Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_ttl: time::Duration::days(7),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthProviderConfig {
    pub name: OAuthProviderName,

    #[serde(flatten)]
    pub config: OAuthProviderClientConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthProvidersConfig {
    /// The list of OAuth providers
    pub providers: Box<[OAuthProviderConfig]>,

    /// The expiration time for the OAuth challenge
    pub challenge_expiration: time::Duration,
}

impl Default for OAuthProvidersConfig {
    fn default() -> Self {
        Self {
            providers: Box::new([]),
            challenge_expiration: time::Duration::minutes(5),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MfaConfig {
    #[serde(default = "default_mfa_token_ttl")]
    pub setup_code_ttl: time::Duration,
    #[serde(default = "default_mfa_token_ttl")]
    pub token_ttl: time::Duration,
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            setup_code_ttl: default_mfa_token_ttl(),
            token_ttl: default_mfa_token_ttl(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthConfig {
    pub email_provider: EmailProviderConfig,
    pub session: SessionConfig,
    pub oauth_providers: OAuthProvidersConfig,

    /// The duration for sudo token to expire
    #[serde(default = "default_sudo_token_ttl")]
    pub sudo_token_ttl: time::Duration,
    pub mfa: MfaConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            sudo_token_ttl: default_sudo_token_ttl(),
            email_provider: EmailProviderConfig::default(),
            session: SessionConfig::default(),
            oauth_providers: OAuthProvidersConfig::default(),
            mfa: MfaConfig::default(),
        }
    }
}

fn default_mfa_token_ttl() -> time::Duration {
    time::Duration::minutes(5)
}

fn default_sudo_token_ttl() -> time::Duration {
    time::Duration::minutes(5)
}

impl admin::utils::config_provider::ConfigJson for AuthConfig {
    const KEY: &'static str = "auth_config";
}
