use clap::{Args, Parser};
use std::fmt;
use url::Url;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// ntfy server URL
    #[arg(env = "NTFY_URL", long, default_value = "https://ntfy.sh", value_parser = parse_ntfy_url)]
    pub ntfy_url: String,

    #[command(flatten)]
    pub auth: AuthConfig,

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
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("ntfy_url", &self.ntfy_url)
            .field("auth", &self.auth)
            .field("api_token", &self.api_token.as_ref().map(|_| "***"))
            .field("listen_addr", &self.listen_addr)
            .field("rate_limit_per_second", &self.rate_limit_per_second)
            .field("rate_limit_burst", &self.rate_limit_burst)
            .finish()
    }
}

#[derive(Args, Clone)]
pub struct AuthConfig {
    #[command(flatten)]
    pub credentials: Credentials,

    /// ntfy access token
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
    /// ntfy username
    #[arg(env = "NTFY_USERNAME", long, requires = "ntfy_password")]
    pub ntfy_username: Option<String>,

    /// ntfy password
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

    pub fn ntfy_token(&self) -> Option<&str> {
        self.auth.ntfy_token.as_deref()
    }

    pub fn ntfy_credentials(&self) -> Option<(&str, &str)> {
        match (
            &self.auth.credentials.ntfy_username,
            &self.auth.credentials.ntfy_password,
        ) {
            (Some(u), Some(p)) => Some((u.as_str(), p.as_str())),
            _ => None,
        }
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

    pub fn load() -> Self {
        dotenvy::dotenv().ok();
        let config = Self::parse();
        tracing::info!("Loaded config: {:?}", config);
        config
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
        assert_eq!(config.ntfy_token(), None);
        assert_eq!(config.ntfy_credentials(), None);
    }

    #[test]
    fn when_parsing_with_token_then_should_set_token() {
        let args = vec!["test", "--ntfy-token", "mytoken"];

        let config = Config::try_parse_from(args).expect("should have valid config with token");

        assert_eq!(config.ntfy_token(), Some("mytoken"));
        assert_eq!(config.ntfy_credentials(), None);
    }

    #[test]
    fn when_parsing_with_credentials_then_should_set_credentials() {
        let args = vec!["test", "--ntfy-username", "user", "--ntfy-password", "pass"];

        let config =
            Config::try_parse_from(args).expect("should have valid config with credentials");

        assert_eq!(config.ntfy_token(), None);
        assert_eq!(config.ntfy_credentials(), Some(("user", "pass")));
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
