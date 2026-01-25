use crate::utils::oauth::providers::{
    ClientCredentialPosition, OAuthDataFetch, OAuthProviderConstants, REQWEST_CLIENT,
};
use crate::utils::oauth::{OAuthUserInfo, OAuthUserInfoError};
use compact_str::CompactString;

#[derive(Debug, Clone, Copy)]
pub struct GoogleProvider;

pub const GOOGLE_CONSTANTS: OAuthProviderConstants = OAuthProviderConstants {
    name: super::OAuthProviderName::Google,
    authorization_url: "https://accounts.google.com/o/oauth2/v2/auth",
    token_url: "https://www.googleapis.com/oauth2/v3/token",
    user_info_url: Some("https://openidconnect.googleapis.com/v1/userinfo"),
    scopes: &[
        "openid",
        "https://www.googleapis.com/auth/userinfo.email",
        "https://www.googleapis.com/auth/userinfo.profile",
    ],
    credential_position: ClientCredentialPosition::InUrl,
    use_pkce: true,
};

impl OAuthDataFetch for GoogleProvider {
    #[tracing::instrument(skip_all, err, fields(provider_name = %GOOGLE_CONSTANTS.name))]
    async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthUserInfoError> {
        let Some(user_info_url) = GOOGLE_CONSTANTS.user_info_url else {
            unreachable!();
        };
        let client = &*REQWEST_CLIENT;
        let response = client
            .get(user_info_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        let google_user: GoogleUserResponse = response.json().await?;

        Ok(OAuthUserInfo {
            id: CompactString::new(google_user.sub),
            email: google_user.email,
            email_verified: google_user.email_verified,
            name: google_user.name,
            picture: google_user.picture,
        })
    }
}

/// Google user information response structure
#[derive(Debug, serde::Deserialize)]
struct GoogleUserResponse {
    sub: String,
    email: Option<CompactString>,
    email_verified: Option<bool>,
    name: Option<CompactString>,
    picture: Option<CompactString>,
}
