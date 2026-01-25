use crate::utils::oauth::providers::{
    ClientCredentialPosition, OAuthDataFetch, OAuthProviderConstants, REQWEST_CLIENT,
};
use crate::utils::oauth::{OAuthUserInfo, OAuthUserInfoError};
use compact_str::CompactString;

#[derive(Debug, Clone, Copy)]
pub struct GithubProvider;

pub const GITHUB_CONSTANTS: OAuthProviderConstants = OAuthProviderConstants {
    name: super::OAuthProviderName::Github,
    authorization_url: "https://github.com/login/oauth/authorize",
    token_url: "https://github.com/login/oauth/access_token",
    user_info_url: Some("https://api.github.com/user"),
    scopes: &["read:user", "user:email"],
    credential_position: ClientCredentialPosition::InUrl,
    use_pkce: false,
};

impl OAuthDataFetch for GithubProvider {
    #[tracing::instrument(skip_all, err, fields(provider_name = %GITHUB_CONSTANTS.name))]
    async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthUserInfoError> {
        let Some(user_info_url) = GITHUB_CONSTANTS.user_info_url else {
            unreachable!();
        };
        let client = &*REQWEST_CLIENT;
        let response = client
            .get(user_info_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        let github_user: GitHubUserResponse = response.json().await?;

        Ok(OAuthUserInfo {
            id: CompactString::new(github_user.id.to_string()),
            email: github_user.email,
            email_verified: None, // GitHub doesn't provide this information
            name: github_user.name,
            picture: github_user.avatar_url,
        })
    }
}

/// GitHub user information response structure
#[derive(Debug, serde::Deserialize)]
struct GitHubUserResponse {
    id: i64,
    email: Option<CompactString>,
    name: Option<CompactString>,
    avatar_url: Option<CompactString>,
}
