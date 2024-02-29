use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph};
use starknet::core::types::SyncStatusType;

use crate::app::App;

pub fn ui(app: &App, frame: &mut Frame) {
    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.size());
    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(outer_layout[0]);

    let sync_area = inner_layout[0];
    let system_area = inner_layout[1];
    let network_area = outer_layout[1];

    render_sync(frame, app, sync_area);
    render_system(frame, app, system_area);
    render_network(frame, app, network_area);
}

fn render_sync(frame: &mut Frame, app: &App, area: Rect) {
    let text = match app.data.syncing.clone() {
        Ok(SyncStatusType::Syncing(status)) => format!(
            "Starting: {} Current: {} Highest: {}",
            status.starting_block_num, status.current_block_num, status.highest_block_num
        ),
        Ok(SyncStatusType::NotSyncing) => format!("Not Syncing"),
        Err(err) => err.clone(),
    };
    frame.render_widget(Block::new().title("Syncing").borders(Borders::ALL), area);
    frame.render_widget(Paragraph::new(format!("{}", text.as_str()).light_green()), area.inner(&Margin::new(2, 1)));
}

fn render_system(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Block::new().title("System").borders(Borders::ALL), area);

    let _outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let cpu_area = area.inner(&Margin::new(1, 1));
    render_cpu(frame, app, cpu_area);
}

fn render_network(frame: &mut Frame, _app: &App, area: Rect) {
    frame.render_widget(Block::new().title("Network").borders(Borders::ALL), area);
}

fn smooth_serie(series: &[f64], window_size: usize) -> Vec<(f64, f64)> {
    let mut smoothed_series = Vec::new();

    let ignore_count = window_size / 2;

    for i in ignore_count..series.len() - ignore_count {
        let window_average: f64 = series[i - ignore_count..=i + ignore_count].iter().sum::<f64>() / window_size as f64;
        smoothed_series.push(window_average);
    }
    let x: Vec<f64> = (0..=100).map(|x| x as f64).collect();
    let serie: Vec<(f64, f64)> = x.into_iter().zip(smoothed_series.into_iter()).collect();
    serie
}

fn render_cpu(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Block::new().title("CPU").borders(Borders::ALL), area);

    let x_labels = vec![];

    let series: Vec<Vec<(f64, f64)>> =
        app.data.cpu_usage.clone().into_iter().map(|elm| smooth_serie(&elm, 7)).take(1).collect();
    let datasets = (0..series.len())
        .map(|index| {
            let serie = &series[index];
            Dataset::default()
                .name("cpu".to_string() + &index.to_string())
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(serie)
        })
        .collect();

    let chart = Chart::new(datasets)
        .block(Block::default().title("CPU usage".cyan().bold()).borders(Borders::ALL))
        .x_axis(
            Axis::default().title("t").style(Style::default().fg(Color::Gray)).labels(x_labels).bounds([0., 100.]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0%".bold(), "50%".bold(), "100%".bold()])
                .bounds([0., 100.]),
        );

    frame.render_widget(chart, area);
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
