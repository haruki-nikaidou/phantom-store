use crate::utils::oauth::providers::{
    ClientCredentialPosition, OAuthDataFetch, OAuthProviderConstants, REQWEST_CLIENT,
};
use crate::utils::oauth::{OAuthUserInfo, OAuthUserInfoError};
use compact_str::CompactString;

#[derive(Debug, Clone, Copy)]
pub struct MicrosoftProvider;

pub const MICROSOFT_CONSTANTS: OAuthProviderConstants = OAuthProviderConstants {
    name: super::OAuthProviderName::Microsoft,
    authorization_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
    token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token",
    user_info_url: Some("https://graph.microsoft.com/oidc/userinfo"),
    scopes: &["openid", "email", "profile", "offline_access"],
    credential_position: ClientCredentialPosition::InBody,
    use_pkce: true,
};

impl OAuthDataFetch for MicrosoftProvider {
    #[tracing::instrument(skip_all, err, fields(provider_name = %MICROSOFT_CONSTANTS.name))]
    async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthUserInfoError> {
        let Some(user_info_url) = MICROSOFT_CONSTANTS.user_info_url else {
            unreachable!();
        };
        let client = &*REQWEST_CLIENT;
        let response = client
            .get(user_info_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        let microsoft_user: MicrosoftUserResponse = response.json().await?;

        Ok(OAuthUserInfo {
            id: microsoft_user.sub,
            email: microsoft_user.email,
            email_verified: microsoft_user.email_verified,
            name: microsoft_user.name,
            picture: microsoft_user.picture,
        })
    }
}

/// Microsoft user information response structure
#[derive(Debug, serde::Deserialize)]
struct MicrosoftUserResponse {
    sub: CompactString,
    email: Option<CompactString>,
    email_verified: Option<bool>,
    name: Option<CompactString>,
    picture: Option<CompactString>,
}
