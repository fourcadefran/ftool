pub mod app;
mod event;
mod terminal;
mod views;
mod widgets;

use app::App;
use std::path::PathBuf;

pub fn run(path: Option<String>) -> anyhow::Result<()> {
    terminal::install_panic_hook();

    let path = path.map(PathBuf::from);
    let mut app = App::new(path)?;
    let mut terminal = terminal::init()?;

    loop {
        terminal.draw(|frame| app.view(frame))?;

        if let Some(ev) = event::poll_event()? {
            let msg = app.handle_event(ev);
            app.update(msg);
        }

        if app.should_quit {
            break;
        }
    }

    terminal::restore()?;
    Ok(())
}
