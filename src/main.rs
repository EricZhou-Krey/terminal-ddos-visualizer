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
    layout::{self, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{
        canvas::{Canvas, Map, MapResolution},
        Block, BorderType, Borders, List, ListItem, Paragraph, Tabs, Widget,
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

    pub fn run(&self, mut terminal: DefaultTerminal) -> Result<(), std::io::Error> {
        loop {
            terminal.draw(|frame| {
                frame.render_widget(self, frame.area());
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

impl Widget for &App {
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
        let request_content = List::new(vec![
            ListItem::new("example1"),
            ListItem::new("exmaple2"),
            ListItem::new("e23"),
        ]);
        request_content.block(request_block).render(
            *layouts.get(&DisplayWidgetLayoutArea::RequestQueue).unwrap(),
            buffer,
        );

        let navbar_block = Block::new().borders(Borders::ALL).title("Navbar");
        let navbar_content = Tabs::new(vec!["Tab1", "Tab2", "Tab3"])
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
    let app = App::new(AppSettings {
        time_interval: TimeDelta::minutes(360),
    });
    let result = app.run(terminal);
    ratatui::restore();
    result
}
