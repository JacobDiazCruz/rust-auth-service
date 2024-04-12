#[derive(Debug, Clone)]
pub struct Config {
    pub google_oauth_client_id: String,
    pub google_oauth_client_secret: String,
    pub google_oauth_redirect_url: String,
    pub google_smtp_username: String,
    pub google_smtp_password: String,
    pub client_url: String,
}

impl Config {
    pub fn init() -> Config {
        let google_oauth_client_id = std::env
            ::var("GOOGLE_OAUTH_CLIENT_ID")
            .expect("GOOGLE_OAUTH_CLIENT_ID must be set");
        let google_oauth_client_secret = std::env
            ::var("GOOGLE_OAUTH_CLIENT_SECRET")
            .expect("GOOGLE_OAUTH_CLIENT_SECRET must be set");
        let google_oauth_redirect_url = std::env
            ::var("GOOGLE_OAUTH_REDIRECT_URL")
            .expect("GOOGLE_OAUTH_REDIRECT_URL must be set");
        let client_url = std::env::var("CLIENT_URL").expect("CLIENT_URL must be set");
        let google_smtp_password = std::env
            ::var("GOOGLE_SMTP_PASSWORD")
            .expect("GOOGLE_SMTP_PASSWORD must be set");
        let google_smtp_username = std::env
            ::var("GOOGLE_SMTP_USERNAME")
            .expect("GOOGLE_SMTP_USERNAME must be set");

        Config {
            google_oauth_client_id,
            google_oauth_client_secret,
            google_oauth_redirect_url,
            client_url,
            google_smtp_password,
            google_smtp_username,
        }
    }
}
