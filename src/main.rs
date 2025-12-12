use std::time::SystemTime;

use chrono::{DateTime, TimeDelta, Utc};
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client,
};

async fn get_content() -> String {
    let token: String = match std::env::var("CLOUDFLARE_API_KEY") {
        Ok(val) => val,
        Err(e) => {
            println!("Cannot find cloudflare API key, {}", e);
            "".to_string()
        }
    };

    let client: Client = Client::new();
    let auth_value =
        HeaderValue::from_str(&format!("Bearer {}", token)).expect("Invalid API Token format");

    let end_time: DateTime<Utc> = SystemTime::now().into();
    let start_time: DateTime<Utc> = end_time - TimeDelta::minutes(30);

    let endpoint_url = format!(
        "https://api.cloudflare.com/client/v4/radar/attacks/layer7/top/attacks?dateStart={}&dateEnd={}",
        start_time.format("%Y-%m-%dT%H:%M:%SZ"),
        end_time.format("%Y-%m-%dT%H:%M:%SZ")
    );

    let response = client
        .get(endpoint_url)
        .header(AUTHORIZATION, auth_value)
        .send()
        .await;

    if let Ok(result) = response {
        result.text().await.unwrap_or("".to_string())
    } else {
        "".to_string()
    }
}

fn main() {
    println!("{}", trpl::block_on(get_content()))
}
