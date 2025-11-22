use crate::models::auth::GitHubUser;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct GitHubOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub org_name: String,
}

impl GitHubOAuthConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            client_id: env::var("GITHUB_CLIENT_ID")
                .map_err(|_| "GITHUB_CLIENT_ID not set".to_string())?,
            client_secret: env::var("GITHUB_CLIENT_SECRET")
                .map_err(|_| "GITHUB_CLIENT_SECRET not set".to_string())?,
            redirect_url: env::var("GITHUB_REDIRECT_URL")
                .map_err(|_| "GITHUB_REDIRECT_URL not set".to_string())?,
            org_name: env::var("GITHUB_ORG_NAME").unwrap_or_else(|_| "amfoss".to_string()),
        })
    }

    pub fn create_client(&self) -> Result<BasicClient, String> {
        let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
            .map_err(|e| format!("Invalid auth URL: {}", e))?;
        let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
            .map_err(|e| format!("Invalid token URL: {}", e))?;

        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new(self.redirect_url.clone())
                .map_err(|e| format!("Invalid redirect URL: {}", e))?,
        );

        Ok(client)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubUserResponse {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubEmailResponse {
    email: String,
    primary: bool,
    verified: bool,
}

pub struct GitHubOAuthService {
    config: GitHubOAuthConfig,
    client: BasicClient,
}

impl GitHubOAuthService {
    pub fn new() -> Result<Self, String> {
        let config = GitHubOAuthConfig::from_env()?;
        let client = config.create_client()?;
        Ok(Self { config, client })
    }

    pub fn get_authorization_url(&self) -> (String, CsrfToken) {
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read:user".to_string()))
            .add_scope(Scope::new("user:email".to_string()))
            .add_scope(Scope::new("read:org".to_string()))
            .url();

        (auth_url.to_string(), csrf_token)
    }

    pub async fn exchange_code(&self, code: String) -> Result<String, String> {
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| format!("Failed to exchange code: {}", e))?;

        Ok(token_result.access_token().secret().clone())
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<GitHubUser, String> {
        let client = reqwest::Client::new();

        let user_response: GitHubUserResponse = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "Root-Backend")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch user info: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse user info: {}", e))?;

        // Fetch from /user/emails if not in profile
        let email = if let Some(email) = user_response.email {
            email
        } else {
            let emails: Vec<GitHubEmailResponse> = client
                .get("https://api.github.com/user/emails")
                .header("Authorization", format!("Bearer {}", access_token))
                .header("User-Agent", "Root-Backend")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch user emails: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse user emails: {}", e))?;

            emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .ok_or("No verified primary email found".to_string())?
                .email
        };

        Ok(GitHubUser {
            github_id: user_response.id,
            github_username: user_response.login,
            name: user_response.name.unwrap_or_else(|| "Unknown".to_string()),
            email,
        })
    }

    pub async fn verify_org_membership(
        &self,
        access_token: &str,
        username: &str,
    ) -> Result<bool, String> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://api.github.com/orgs/{}/members/{}",
            self.config.org_name, username
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "Root-Backend")
            .send()
            .await
            .map_err(|e| format!("Failed to check org membership: {}", e))?;

        // GitHub returns 204 if member, 404 if not, 302 if needs authentication
        Ok(response.status().as_u16() == 204)
    }

    pub async fn complete_oauth_flow(&self, code: String) -> Result<GitHubUser, String> {
        let access_token = self.exchange_code(code).await?;

        let user_info = self.get_user_info(&access_token).await?;

        let is_member = self
            .verify_org_membership(&access_token, &user_info.github_username)
            .await?;

        if !is_member {
            return Err(format!(
                "User is not a member of the {} organization",
                self.config.org_name
            ));
        }

        Ok(user_info)
    }
}
