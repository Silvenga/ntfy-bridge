use clap::{Args, Parser};
use std::fmt;
use url::Url;

#[derive(Clone)]
pub enum NtfyCredentials {
    None,
    AuthToken(String),
    UsernamePassword(String, String),
}

impl fmt::Display for NtfyCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::AuthToken(_) => write!(f, "AuthToken(***)"),
            Self::UsernamePassword(u, _) => write!(f, "UsernamePassword({}, ***)", u),
        }
    }
}

impl fmt::Debug for NtfyCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Ntfy server URL
    #[arg(env = "NTFY_URL", long, default_value = "https://ntfy.sh", value_parser = parse_ntfy_url)]
    pub ntfy_url: String,

    #[command(flatten)]
    pub ntfy_auth: AuthConfig,

    /// API token for authentication
    #[arg(env = "API_TOKEN", long)]
    pub api_token: Option<String>,

    /// Address to listen on
    #[arg(env = "LISTEN_ADDR", long, default_value = "0.0.0.0:8080")]
    pub listen_addr: String,

    /// Rate limit requests per second
    #[arg(env = "RATE_LIMIT_PER_SECOND", long, default_value = "2")]
    pub rate_limit_per_second: u64,

    /// Rate limit burst size
    #[arg(env = "RATE_LIMIT_BURST", long, default_value = "5")]
    pub rate_limit_burst: u32,

    /// Trust X-Forwarded-For headers from a reverse proxy
    #[arg(env = "USE_X_FORWARDED_FOR", long, default_value_t = false)]
    pub use_x_forwarded_for: bool,

    /// Log level
    #[arg(env = "LOG_LEVEL", long, default_value = "info")]
    pub log_level: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("ntfy_url", &self.ntfy_url)
            .field("ntfy_auth", &self.ntfy_credentials())
            .field("api_token", &self.api_token.as_ref().map(|_| "***"))
            .field("listen_addr", &self.listen_addr)
            .field("rate_limit_per_second", &self.rate_limit_per_second)
            .field("rate_limit_burst", &self.rate_limit_burst)
            .field("use_x_forwarded_for", &self.use_x_forwarded_for)
            .field("log_level", &self.log_level)
            .finish()
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Args, Clone)]
pub struct AuthConfig {
    #[command(flatten)]
    pub credentials: Credentials,

    /// Ntfy access token
    #[arg(env = "NTFY_TOKEN", long, conflicts_with = "credentials")]
    pub ntfy_token: Option<String>,
}

impl fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthConfig")
            .field("credentials", &self.credentials)
            .field("ntfy_token", &self.ntfy_token.as_ref().map(|_| "***"))
            .finish()
    }
}

#[derive(Args, Clone)]
#[group(id = "credentials", multiple = true)]
pub struct Credentials {
    /// Ntfy username
    #[arg(env = "NTFY_USERNAME", long, requires = "ntfy_password")]
    pub ntfy_username: Option<String>,

    /// Ntfy password
    #[arg(env = "NTFY_PASSWORD", long, requires = "ntfy_username")]
    pub ntfy_password: Option<String>,
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("ntfy_username", &self.ntfy_username)
            .field("ntfy_password", &self.ntfy_password.as_ref().map(|_| "***"))
            .finish()
    }
}

impl Config {
    pub fn ntfy_url(&self) -> &str {
        &self.ntfy_url
    }

    pub fn ntfy_credentials(&self) -> NtfyCredentials {
        if let Some(token) = &self.ntfy_auth.ntfy_token {
            return NtfyCredentials::AuthToken(token.clone());
        }

        if let (Some(username), Some(password)) = (
            &self.ntfy_auth.credentials.ntfy_username,
            &self.ntfy_auth.credentials.ntfy_password,
        ) {
            return NtfyCredentials::UsernamePassword(username.clone(), password.clone());
        }

        NtfyCredentials::None
    }

    pub fn listen_addr(&self) -> &str {
        &self.listen_addr
    }

    pub fn api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }

    pub fn rate_limit_per_second(&self) -> u64 {
        self.rate_limit_per_second
    }

    pub fn rate_limit_burst(&self) -> u32 {
        self.rate_limit_burst
    }

    pub fn use_x_forwarded_for(&self) -> bool {
        self.use_x_forwarded_for
    }

    pub fn log_level(&self) -> &str {
        &self.log_level
    }

    pub fn load() -> Self {
        dotenvy::dotenv().ok();
        Self::parse()
    }
}

fn parse_ntfy_url(s: &str) -> Result<String, String> {
    let url = Url::parse(s).map_err(|e| format!("invalid URL: {}", e))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("scheme must be http or https".to_string());
    }

    if url.host().is_none() {
        return Err("URL must have a host".to_string());
    }

    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_parsing_with_defaults_then_should_have_correct_values() {
        let args: Vec<String> = vec!["test".to_string()];

        let config = Config::try_parse_from(args).expect("should have valid default config");

        assert_eq!(config.ntfy_url(), "https://ntfy.sh");
        assert_eq!(config.listen_addr(), "0.0.0.0:8080");
        match config.ntfy_credentials() {
            NtfyCredentials::None => {}
            _ => panic!("should have no credentials"),
        }
        assert!(!config.use_x_forwarded_for());
    }

    #[test]
    fn when_parsing_with_x_forwarded_for_then_should_set_it() {
        let args = vec!["test", "--use-x-forwarded-for"];

        let config =
            Config::try_parse_from(args).expect("should have valid config with x-forwarded-for");

        assert!(config.use_x_forwarded_for());
    }

    #[test]
    fn when_parsing_with_token_then_should_set_token() {
        let args = vec!["test", "--ntfy-token", "mytoken"];

        let config = Config::try_parse_from(args).expect("should have valid config with token");

        match config.ntfy_credentials() {
            NtfyCredentials::AuthToken(token) => assert_eq!(token, "mytoken"),
            _ => panic!("should have token credentials"),
        }
    }

    #[test]
    fn when_parsing_with_credentials_then_should_set_credentials() {
        let args = vec!["test", "--ntfy-username", "user", "--ntfy-password", "pass"];

        let config =
            Config::try_parse_from(args).expect("should have valid config with credentials");

        match config.ntfy_credentials() {
            NtfyCredentials::UsernamePassword(u, p) => {
                assert_eq!(u, "user");
                assert_eq!(p, "pass");
            }
            _ => panic!("should have username/password credentials"),
        }
    }

    #[test]
    fn when_parsing_with_token_and_credentials_then_should_fail() {
        let args = vec![
            "test",
            "--ntfy-token",
            "mytoken",
            "--ntfy-username",
            "user",
            "--ntfy-password",
            "pass",
        ];

        let result = Config::try_parse_from(args);

        assert!(result.is_err());
    }

    #[test]
    fn when_parsing_with_url_then_should_set_url() {
        let args = vec!["test", "--ntfy-url", "https://example.com"];

        let config = Config::try_parse_from(args).expect("should have valid config with url");

        assert_eq!(config.ntfy_url(), "https://example.com");
    }

    #[test]
    fn when_parsing_with_listen_addr_then_should_set_listen_addr() {
        let args = vec!["test", "--listen-addr", "127.0.0.1:9090"];

        let config =
            Config::try_parse_from(args).expect("should have valid config with listen address");

        assert_eq!(config.listen_addr(), "127.0.0.1:9090");
    }

    #[test]
    fn when_parsing_invalid_url_then_should_fail() {
        let args = vec!["test", "--ntfy-url", "not-a-url"];
        let result = Config::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn when_parsing_ftp_url_then_should_fail() {
        let args = vec!["test", "--ntfy-url", "ftp://ntfy.sh"];
        let result = Config::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn when_parsing_url_without_host_then_should_fail() {
        let args = vec!["test", "--ntfy-url", "https://"];
        let result = Config::try_parse_from(args);
        assert!(result.is_err());
    }
}
