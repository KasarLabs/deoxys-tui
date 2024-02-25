use anyhow::Result;
use crossterm::{
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
  layout::{Constraint, Direction, Layout, Margin}, prelude::Frame, style::Stylize, widgets::{Block, Borders, Paragraph}
};
use starknet::core::types::SyncStatusType;
use crate::app::App;

pub fn ui(app: &App, frame: &mut Frame) {
  let outer_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(vec![
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(frame.size());
  let inner_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(vec![
        Constraint::Percentage(25),
        Constraint::Percentage(75),
    ])
    .split(outer_layout[0]);

  frame.render_widget(Block::new().title("Syncing").borders(Borders::ALL), outer_layout[0]);
  frame.render_widget(Block::new().title("System").borders(Borders::ALL), outer_layout[1]);
  frame.render_widget(Paragraph::new(format!("{}", syncing_widget(&app.data.syncing).as_str()).light_green()), outer_layout[0].inner(&Margin::new(2, 1)));
  frame.render_widget(Block::new().title("Network").borders(Borders::ALL), inner_layout[1]);
}

fn syncing_widget(sync: &SyncStatusType) -> String {
  match sync {
    SyncStatusType::Syncing(status) => format!("starting_block_num: {} current_block_num: {} highest_block_num:{}", status.starting_block_num, status.current_block_num, status.highest_block_num),
    SyncStatusType::NotSyncing => format!("Not Syncing")
  }
}

pub fn startup() -> Result<()> {
  enable_raw_mode()?;
  execute!(std::io::stderr(), EnterAlternateScreen)?;
  Ok(())
}

pub fn shutdown() -> Result<()> {
  execute!(std::io::stderr(), LeaveAlternateScreen)?;
  disable_raw_mode()?;
  Ok(())
}
