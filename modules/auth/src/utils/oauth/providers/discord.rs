use crate::utils::oauth::providers::{
    ClientCredentialPosition, OAuthDataFetch, OAuthProviderConstants, REQWEST_CLIENT,
};
use crate::utils::oauth::{OAuthUserInfo, OAuthUserInfoError};
use compact_str::{CompactString, format_compact};

#[derive(Debug, Clone, Copy)]
pub struct DiscordProvider;

pub const DISCORD_CONSTANTS: OAuthProviderConstants = OAuthProviderConstants {
    name: super::OAuthProviderName::Discord,
    authorization_url: "https://discord.com/api/oauth2/authorize",
    token_url: "https://discord.com/api/oauth2/token",
    user_info_url: Some("https://discord.com/api/users/@me"),
    scopes: &["identify", "email"],
    credential_position: ClientCredentialPosition::InUrl,
    use_pkce: false,
};

impl OAuthDataFetch for DiscordProvider {
    #[tracing::instrument(skip_all, err, fields(provider_name = %DISCORD_CONSTANTS.name))]
    async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthUserInfoError> {
        let Some(user_info_url) = DISCORD_CONSTANTS.user_info_url else {
            unreachable!();
        };
        let client = &*REQWEST_CLIENT;
        let response = client
            .get(user_info_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        let discord_user: DiscordUserResponse = response.json().await?;

        // Construct avatar URL if available
        let picture = discord_user.avatar.map(|avatar| {
            format_compact!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                discord_user.id,
                avatar
            )
        });

        // Prefer global_name over username if available
        let name = discord_user
            .global_name
            .or(discord_user.username)
            .map(CompactString::new);

        Ok(OAuthUserInfo {
            id: CompactString::new(discord_user.id),
            email: discord_user.email.map(CompactString::new),
            email_verified: discord_user.verified,
            name,
            picture,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct DiscordUserResponse {
    id: String,
    email: Option<String>,
    verified: Option<bool>,
    username: Option<String>,
    global_name: Option<String>,
    avatar: Option<String>,
}
