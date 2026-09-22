use reqwest::Client;
use std::time::Duration;

pub fn build_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent("KTUN-Bot/2.0")
        .timeout(Duration::from_secs(15))
        .build()
}
