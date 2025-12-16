use std::{collections::VecDeque, time::SystemTime};

use chrono::{DateTime, TimeDelta, Utc};
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client,
};

use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct CloudflareResponse {
    success: bool,
    errors: Vec<String>,
    result: CloudflareDDOSAttackResult,
}

#[derive(Deserialize, Debug)]
struct CloudflareDDOSAttackResult {
    top_0: VecDeque<DDOSAttack>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DDOSAttack {
    origin_country_alpha2: String,
    origin_country_name: String,
    target_country_alpha2: String,
    target_country_name: String,
}

impl DDOSAttack {
    pub fn get_content(&self) -> (&str, &str) {
        (&self.origin_country_name, &self.target_country_name)
    }

    pub fn get_codes(&self) -> (&str, &str) {
        (&self.origin_country_alpha2, &self.target_country_alpha2)
    }
}

pub trait DDOSProvider {
    async fn get_ddos_attacks(
        &mut self,
        time_interval: TimeDelta,
    ) -> Result<VecDeque<DDOSAttack>, &'static str>;
}

#[derive(Debug)]
pub struct CloudflareDDOSCompoent {}

impl CloudflareDDOSCompoent {
    pub fn new() -> Self {
        Self {}
    }
}

impl CloudflareDDOSCompoent {
    async fn cloudflare_ddos(&self, time_interval: TimeDelta) -> CloudflareResponse {
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
        let start_time: DateTime<Utc> = end_time - time_interval;

        let endpoint_url = format!(
            "https://api.cloudflare.com/client/v4/radar/attacks/layer7/top/attacks?dateStart={}&dateEnd={}",
            start_time.format("%Y-%m-%dT%H:%M:%SZ"),
            end_time.format("%Y-%m-%dT%H:%M:%SZ")
        );

        client
            .get(endpoint_url)
            .header(AUTHORIZATION, auth_value)
            .send()
            .await
            .unwrap()
            .json::<CloudflareResponse>()
            .await
            .unwrap()
    }
}

impl DDOSProvider for CloudflareDDOSCompoent {
    async fn get_ddos_attacks(
        &mut self,
        time_interval: TimeDelta,
    ) -> Result<VecDeque<DDOSAttack>, &'static str> {
        let cloudflare_response = self.cloudflare_ddos(time_interval).await;

        if cloudflare_response.success {
            Ok(cloudflare_response.result.top_0)
        } else {
            Err("Cloudflare Error")
        }
    }
}
