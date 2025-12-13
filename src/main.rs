use std::{collections::VecDeque, time::SystemTime};

use chrono::{DateTime, TimeDelta, Utc};
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client,
};

use serde::Deserialize;

use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};
//
// Will decouple cloudflare reqwesting into library file later
//

#[derive(Deserialize, Debug)]
struct CloudflareResponse {
    success: bool,
    errors: Vec<String>,
    result: CloudflareDDOSAttackResult,
}

#[derive(Deserialize, Debug)]
struct CloudflareDDOSAttackResult {
    top_0: Vec<CloudflareAttack>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CloudflareAttack {
    origin_country_alpha2: String,
    origin_country_name: String,
    target_country_alpha2: String,
    target_country_name: String,
}

async fn get_content() -> CloudflareResponse {
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
    let start_time: DateTime<Utc> = end_time - TimeDelta::minutes(360);

    let endpoint_url = format!(
        "https://api.cloudflare.com/client/v4/radar/attacks/layer7/top/attacks?dateStart={}&dateEnd={}",
        start_time.format("%Y-%m-%dT%H:%M:%SZ"),
        end_time.format("%Y-%m-%dT%H:%M:%SZ")
    );

    // Bunch of errors I am ignoring for now

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

#[derive(Debug, Default)]
enum Region {
    #[default]
    World,
    Europe,
    Asia,
    Australasia,
    NorthAmerica,
    SouthAmerica,
    Africa,
}

#[derive(Debug, Default)]
struct AppSettings {
    time_interval: TimeDelta,
}

#[derive(Debug, Default)]
struct App {
    map_region_display: Region,
    cloudflare_attack_queue: VecDeque<CloudflareAttack>,
    settings: AppSettings,
}

impl App {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            map_region_display: Region::World,
            cloudflare_attack_queue: VecDeque::new(),
            settings,
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<(), std::io::Error> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello_world", frame.area());
}
