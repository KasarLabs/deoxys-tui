use crossterm::event::{self, Event::Key, KeyCode::Char};
use ratatui::prelude::{CrosstermBackend, Terminal};
use anyhow::{Ok, Result};
use crate::{app::App, ui};

pub async fn run(rpc_endpoint: &str) -> Result<()> {
    ui::startup()?;

    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut app = App::new("Deoxys Dashboard", rpc_endpoint);

    loop {
        update(&mut app).await?;
        t.draw(|f| {
            ui::ui(&app, f);
        })?;
        if app.should_quit {
            break;
        }
    }
    ui::shutdown()?;
    Ok(())
}

async fn update(app: &mut App) -> Result<()> {
    app.update_metrics().await;
    if event::poll(std::time::Duration::from_millis(250))? {
      if let Key(key) = event::read()? {
        if key.kind == event::KeyEventKind::Press {
          match key.code {
            Char('q') => app.should_quit = true,
            _ => {},
          }
        }
      }
    }
    Ok(())
}
  