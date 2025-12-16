use std::{
    collections::{HashMap, VecDeque},
    time::SystemTime,
};

use chrono::{DateTime, TimeDelta, Utc};
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client,
};

use serde::Deserialize;

use crossterm::event::{self, Event};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Map, MapResolution},
        Block, Borders, List, Paragraph, Tabs, Widget,
    },
    DefaultTerminal,
};

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
    top_0: VecDeque<DDOSAttack>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DDOSAttack {
    origin_country_alpha2: String,
    origin_country_name: String,
    target_country_alpha2: String,
    target_country_name: String,
}

trait DDOSProvider {
    async fn get_ddos_attacks(
        &mut self,
        time_interval: TimeDelta,
    ) -> Result<VecDeque<DDOSAttack>, &'static str>;
}

#[derive(Debug)]
struct CloudflareDDOSCompoent {}

impl CloudflareDDOSCompoent {
    fn new() -> Self {
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

#[derive(Debug, Default)]
enum Region {
    #[default]
    World,
    Europe,
    Asia,
    Oceania,
    NorthAmerica,
    SouthAmerica,
    Africa,
}

#[derive(Debug)]
struct AppSettings {
    current_region: Region,
    time_interval: TimeDelta,
}

impl AppSettings {
    fn new() -> Self {
        Self {
            current_region: Region::default(),
            time_interval: TimeDelta::minutes(360),
        }
    }
}
#[derive(Debug)]
struct App<T: DDOSProvider> {
    ddos_componet: T,
    ddos_attack_queue: VecDeque<DDOSAttack>,
    settings: AppSettings,
}

impl App<CloudflareDDOSCompoent> {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            ddos_componet: CloudflareDDOSCompoent::new(),
            ddos_attack_queue: VecDeque::new(),
            settings,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), std::io::Error> {
        loop {
            if self.ddos_attack_queue.is_empty() {
                self.ddos_attack_queue = trpl::block_on(
                    self.ddos_componet
                        .get_ddos_attacks(self.settings.time_interval),
                )
                .unwrap_or_else(|_| VecDeque::new());
            }
            let _ = terminal.draw(|frame| {
                frame.render_widget(&self, frame.area());
            });
            if matches!(event::read()?, Event::Key(_)) {
                break Ok(());
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
enum DisplayWidgetLayoutArea {
    Navbar,
    RequestQueue,
    Map,
    Settings,
}

impl<T: DDOSProvider> Widget for &App<T> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let layouts: HashMap<DisplayWidgetLayoutArea, Rect> = {
            let display_settings_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(area);

            let navdata_map_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(display_settings_layout[0]);

            let nav_data_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
                .split(navdata_map_layout[0]);

            HashMap::from([
                (DisplayWidgetLayoutArea::Navbar, nav_data_layout[0]),
                (DisplayWidgetLayoutArea::RequestQueue, nav_data_layout[1]),
                (DisplayWidgetLayoutArea::Map, navdata_map_layout[1]),
                (
                    DisplayWidgetLayoutArea::Settings,
                    display_settings_layout[1],
                ),
            ])
        };

        let settings_block = Block::new().borders(Borders::ALL).title("Settings");
        let settings_content = Paragraph::new(format!(
            "Current Time Interval = {} minutes",
            self.settings.time_interval.num_minutes()
        ));
        settings_content.block(settings_block).render(
            *layouts.get(&DisplayWidgetLayoutArea::Settings).unwrap(),
            buffer,
        );

        let map_block = Block::new().borders(Borders::ALL).title("Map");
        let map_content = Canvas::default()
            .marker(Marker::Braille)
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Green,
                    resolution: MapResolution::High,
                })
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        map_content
            .block(map_block)
            .render(*layouts.get(&DisplayWidgetLayoutArea::Map).unwrap(), buffer);

        let request_block = Block::new().borders(Borders::ALL).title("RequestQueue");
        let request_content = List::new(self.ddos_attack_queue.iter().map(|ddos_attack| {
            format!(
                "from {}, to {}",
                ddos_attack.origin_country_name, ddos_attack.target_country_name
            )
        }));
        request_content.block(request_block).render(
            *layouts.get(&DisplayWidgetLayoutArea::RequestQueue).unwrap(),
            buffer,
        );

        let navbar_block = Block::new().borders(Borders::ALL).title("Navbar");
        let navbar_content = Tabs::new(vec![
            "World",
            "Europe",
            "Asia",
            "Oceania",
            "N. America",
            "S. America",
            "Africa",
        ])
        .select(0)
        .padding("", "")
        .divider(" ");
        navbar_content.block(navbar_block).render(
            *layouts.get(&DisplayWidgetLayoutArea::Navbar).unwrap(),
            buffer,
        );
    }
}

fn main() -> Result<(), std::io::Error> {
    let terminal = ratatui::init();
    let app = App::new(AppSettings::new());
    let result = app.run(terminal);
    ratatui::restore();
    result
}
