use terminal_ddos_visualizer::app::{App, AppSettings};
fn main() -> Result<(), std::io::Error> {
    let terminal = ratatui::init();
    let app = App::new(AppSettings::new());
    let result = app.run(terminal);
    ratatui::restore();
    result
}
