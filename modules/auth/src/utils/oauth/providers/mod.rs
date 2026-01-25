use crate::entities::redis::oauth_challenge::OAuthChallengeKey;
use crate::utils::oauth::client_config::OAuthProviderClientConfig;
use crate::utils::oauth::{OAuthUserInfo, OAuthUserInfoError};

pub mod discord;
pub mod github;
pub mod google;
pub mod microsoft;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
/// All providers supported
pub enum OAuthProviderName {
    Google,
    Microsoft,
    Github,
    Discord,
}

impl OAuthProviderName {
    pub fn oauth_constants(&self) -> OAuthProviderConstants {
        match self {
            OAuthProviderName::Google => google::GOOGLE_CONSTANTS,
            OAuthProviderName::Microsoft => microsoft::MICROSOFT_CONSTANTS,
            OAuthProviderName::Github => github::GITHUB_CONSTANTS,
            OAuthProviderName::Discord => discord::DISCORD_CONSTANTS,
        }
    }
}

impl OAuthDataFetch for OAuthProviderName {
    async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthUserInfoError> {
        match self {
            OAuthProviderName::Google => google::GoogleProvider.fetch_user_info(access_token).await,
            OAuthProviderName::Microsoft => {
                microsoft::MicrosoftProvider
                    .fetch_user_info(access_token)
                    .await
            }
            OAuthProviderName::Github => github::GithubProvider.fetch_user_info(access_token).await,
            OAuthProviderName::Discord => {
                discord::DiscordProvider.fetch_user_info(access_token).await
            }
        }
    }
}

impl std::fmt::Display for OAuthProviderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProviderName::Google => write!(f, "google"),
            OAuthProviderName::Microsoft => write!(f, "microsoft"),
            OAuthProviderName::Github => write!(f, "github"),
            OAuthProviderName::Discord => write!(f, "discord"),
        }
    }
}

impl std::str::FromStr for OAuthProviderName {
    type Err = framework::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "google" => Ok(OAuthProviderName::Google),
            "microsoft" => Ok(OAuthProviderName::Microsoft),
            "github" => Ok(OAuthProviderName::Github),
            "discord" => Ok(OAuthProviderName::Discord),
            _ => Err(framework::Error::InvalidInput),
        }
    }
}

pub(crate) trait OAuthDataFetch {
    async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthUserInfoError>;
}

#[allow(clippy::type_complexity)]
type OAuthClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;

lazy_static::lazy_static! {
    static ref REQWEST_CLIENT: reqwest::Client = {
        // SAFETY: it will never fail
        #[allow(clippy::expect_used)]
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .gzip(true)
            .brotli(true)
            .zstd(true)
            .deflate(true)
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build request sender")
    };
}

pub(crate) const USER_AGENT: &str = "helium-microservices/0.1.0";

type OAuthCodeExchangeResponse =
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>;
type OAuthCodeExchangeError = oauth2::RequestTokenError<
    oauth2::HttpClientError<reqwest::Error>,
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
>;

#[derive(Debug, Clone)]
pub struct AuthorizeUrlEssentials {
    pub state: OAuthChallengeKey,
    pub authorization_url: url::Url,
    pub pkce_verifier: Option<String>,
}

impl OAuthProviderConstants {
    /// Builds an OAuth client for the provider
    #[tracing::instrument(skip_all, err, fields(provider_name = %self.name))]
    pub fn build_oauth_client(
        &self,
        config: &OAuthProviderClientConfig,
        redirect_uri: &str,
    ) -> Result<OAuthClient, url::ParseError> {
        let client_id = oauth2::ClientId::new(config.client_id.to_string());
        let client_secret = oauth2::ClientSecret::new(config.client_secret.to_string());
        let auth_url = oauth2::AuthUrl::new(self.authorization_url.to_owned())?;
        let token_url = oauth2::TokenUrl::new(self.token_url.to_owned())?;
        let redirect_uri = oauth2::RedirectUrl::new(redirect_uri.to_owned())?;
        let client: OAuthClient = oauth2::basic::BasicClient::new(client_id)
            .set_client_secret(client_secret)
            .set_auth_type(match self.credential_position {
                ClientCredentialPosition::InBody => oauth2::AuthType::RequestBody,
                ClientCredentialPosition::InUrl => oauth2::AuthType::BasicAuth,
            })
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(redirect_uri);
        Ok(client)
    }

    #[tracing::instrument(skip_all, fields(provider_name = %self.name))]
    pub fn full_authorize_url(&self, client: &OAuthClient) -> AuthorizeUrlEssentials {
        let new_state = OAuthChallengeKey::generate();
        let pkce_verifier = if self.use_pkce {
            Some(oauth2::PkceCodeChallenge::new_random_sha256())
        } else {
            None
        };
        let mut req = client.authorize_url(|| oauth2::CsrfToken::new(new_state.to_string()));
        for scope in self.scopes {
            req = req.add_scope(oauth2::Scope::new((*scope).to_owned()));
        }
        if let Some((pkce_challenge, pkce_verifier)) = pkce_verifier {
            req = req.set_pkce_challenge(pkce_challenge);
            let (url, _) = req.url();
            AuthorizeUrlEssentials {
                authorization_url: url,
                state: new_state,
                pkce_verifier: Some(pkce_verifier.into_secret()),
            }
        } else {
            let (url, _) = req.url();
            AuthorizeUrlEssentials {
                authorization_url: url,
                state: new_state,
                pkce_verifier: None,
            }
        }
    }

    #[tracing::instrument(skip_all, err, fields(provider_name = %self.name))]
    pub async fn exchange_code(
        &self,
        client: &OAuthClient,
        code: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<OAuthCodeExchangeResponse, OAuthCodeExchangeError> {
        let mut req = client.exchange_code(oauth2::AuthorizationCode::new(code.to_owned()));
        if let Some(pkce_verifier) = pkce_verifier {
            req = req.set_pkce_verifier(oauth2::PkceCodeVerifier::new(pkce_verifier.to_owned()));
        }
        req.request_async(&*REQWEST_CLIENT).await
    }
}

#[derive(Debug, Clone)]
pub struct OAuthProviderConstants {
    name: OAuthProviderName,
    authorization_url: &'static str,
    token_url: &'static str,
    user_info_url: Option<&'static str>,
    scopes: &'static [&'static str],
    credential_position: ClientCredentialPosition,
    use_pkce: bool,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ClientCredentialPosition {
    /// Set the client credentials in the request body
    ///
    /// Microsoft use this method.
    InBody,
    InUrl,
}
