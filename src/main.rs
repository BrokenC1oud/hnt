use color_eyre::Result;
use hnt::ui::App;

fn main() -> Result<()> {
    let mut app = App::new();
    let mut terminal = ratatui::init();
    app.run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}
