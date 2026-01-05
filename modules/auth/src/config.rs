use crate::entities::redis::session::SessionId;
use crate::utils::jwt::{AccessToken, RefreshToken, TokenClaims};
use crate::utils::oauth::client_config::OAuthProviderClientConfig;
use crate::utils::oauth::providers::OAuthProviderName;
use compact_str::CompactString;
use uuid::Uuid;

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
pub struct JwtConfig {
    pub secret: CompactString,
    pub issuer: CompactString,
    pub audience: CompactString,

    /// Duration of refresh token validity and the TTL of session in redis.
    pub refresh_token_ttl: time::Duration,

    /// Duration of access token validity. If the access token is expired, the client
    /// needs to use the refresh token to get a new access token and refresh token.
    pub access_token_ttl: time::Duration,
}

impl JwtConfig {
    const ALGORITHM: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS512;
    pub fn encoder(&self) -> impl Fn(TokenClaims) -> jsonwebtoken::errors::Result<String> {
        let key = jsonwebtoken::EncodingKey::from_secret(self.secret.as_ref());
        let header = jsonwebtoken::Header::new(Self::ALGORITHM);
        move |claims| jsonwebtoken::encode(&header, &claims, &key)
    }
    pub fn decoder(
        &self,
    ) -> impl Fn(&str) -> jsonwebtoken::errors::Result<jsonwebtoken::TokenData<TokenClaims>> {
        let key = jsonwebtoken::DecodingKey::from_secret(self.secret.as_ref());
        let mut validation = jsonwebtoken::Validation::new(Self::ALGORITHM);
        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&[&self.issuer]);
        let validation = validation;
        move |token| jsonwebtoken::decode(token, &key, &validation)
    }
    pub fn refresh_token_decoder(
        &self,
    ) -> impl Fn(&str) -> jsonwebtoken::errors::Result<jsonwebtoken::TokenData<TokenClaims>> {
        let key = jsonwebtoken::DecodingKey::from_secret(self.secret.as_ref());
        let mut validation = jsonwebtoken::Validation::new(Self::ALGORITHM);
        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&[&self.issuer]);
        let validation = validation;
        move |token| jsonwebtoken::decode(token, &key, &validation)
    }
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        session_id: SessionId,
    ) -> Result<AccessToken, jsonwebtoken::errors::Error> {
        let encode = self.encoder();
        let exp =
            (time::OffsetDateTime::now_utc() + self.access_token_ttl).unix_timestamp() as usize;
        let claims = TokenClaims {
            sub: user_id,
            sid: session_id.0,
            exp,
            iss: self.issuer,
            aud: self.audience,
        };
        let token_str = encode(claims)?;
        Ok(AccessToken::new(token_str))
    }

    pub fn generate_refresh_token(
        &self,
        user_id: Uuid,
        session_id: SessionId,
    ) -> Result<RefreshToken, jsonwebtoken::errors::Error> {
        let encode = self.encoder();
        let exp =
            (time::OffsetDateTime::now_utc() + self.refresh_token_ttl).unix_timestamp() as usize;
        let claims = TokenClaims {
            sub: user_id,
            sid: session_id.0,
            exp,
            iss: self.issuer,
            aud: self.audience,
        };
        let token_str = encode(claims)?;
        Ok(RefreshToken::new(token_str))
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        use rand::distr::SampleString;
        let random_secret = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 32);
        tracing::warn!(
            "JWT Secret is not set, using default config and the secret is randomly generated. If you see this warning during the initial setup, you can ignore it. Otherwise, please set a proper secret in the configuration."
        );
        Self {
            secret: CompactString::new(&random_secret),
            issuer: CompactString::new("phantom_store"),
            audience: CompactString::new("phantom_store_user"),
            refresh_token_ttl: time::Duration::days(7),
            access_token_ttl: time::Duration::minutes(15),
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
pub struct AuthConfig {
    pub email_provider: EmailProviderConfig,
    pub jwt: JwtConfig,
    pub oauth_providers: OAuthProvidersConfig,

    /// The duration for sudo token to expire
    #[serde(default = "default_sudo_token_ttl")]
    pub sudo_token_ttl: time::Duration,

    /// The duration for MFA token to expire
    #[serde(default = "default_mfa_token_ttl")]
    pub mfa_token_ttl: time::Duration,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            sudo_token_ttl: default_sudo_token_ttl(),
            email_provider: EmailProviderConfig::default(),
            jwt: JwtConfig::default(),
            oauth_providers: OAuthProvidersConfig::default(),
            mfa_token_ttl: default_mfa_token_ttl(),
        }
    }
}

fn default_mfa_token_ttl() -> time::Duration {
    time::Duration::minutes(5)
}

fn default_sudo_token_ttl() -> time::Duration {
    time::Duration::minutes(5)
}
