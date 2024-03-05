use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, Paragraph};
use starknet::core::types::SyncStatusType;

use crate::app::App;

pub fn ui(app: &App, frame: &mut Frame) {
    let node0 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.size());

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(33); 3])
        .split(node0[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(33); 3])
        .split(node0[1]);
    render_cpu(frame, app, right[0]);
    render_memory(frame, app, right[1]);
    render_storage(frame, app, right[2]);
    render_l1_logs(frame, app, left[0]);
    render_l2_logs(frame, app, left[1]);
    render_rpc_playground(frame, app, left[2]);
}

fn _render_sync(frame: &mut Frame, app: &App, area: Rect) {
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

fn render_zone(frame: &mut Frame, area: Rect, title: &str) {
    let outline = Block::new().borders(Borders::ALL).title(title);
    frame.render_widget(outline, area);
}

fn render_storage(frame: &mut Frame, app: &App, area: Rect) { //SALE: découer et simplifier
    let color;
    let ratio = app.data.disk_usage as f64 / app.data.disk_size as f64;
    if ratio <= 1. / 3. {
        color = Color::Green;
    } else if ratio <= 2. / 3. {
        color = Color::Rgb(255, 128, 0)
    } else {
        color = Color::Red;
    }

    let zone = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);
    render_zone(frame, area, "Storage");

    let gauge = Gauge::default()
        .gauge_style(color)
        .bg(Color::Rgb(10, 10, 10))
        .ratio(ratio)
        .block(Block::default().borders(Borders::ALL).title("Used").title_alignment(Alignment::Center));
    frame.render_widget(gauge, zone[1].inner(&Margin::new(1, 1)));

    let test = Paragraph::new(format!(
        "Total: {} Go | Available: {} Go | Used: {} Go",
        app.data.disk_size / 1000000000,
        app.data.available_storage / 1000000000,
        app.data.disk_usage / 1000000000
    ))
    .left_aligned();

    frame.render_widget(test, zone[0].inner(&Margin::new(1, 1)));
}

fn render_l1_logs(frame: &mut Frame, app: &App, area: Rect) {
    render_zone(frame, area, "L1 logs")
}

fn render_l2_logs(frame: &mut Frame, app: &App, area: Rect) {
    render_zone(frame, area, "L2 logs")
}

fn render_rpc_playground(frame: &mut Frame, app: &App, area: Rect) {
    render_zone(frame, area, "RPC Playground")
}

fn render_cpu(frame: &mut Frame, app: &App, area: Rect) {
    let x_labels = vec![];

    let serie = smooth_serie(&app.data.cpu_usage, 7);

    let datasets = vec![
        Dataset::default()
            .name("cpu")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(&serie),
    ];

    let chart = Chart::new(datasets)
        .block(Block::default().title("CPU".cyan().bold()).borders(Borders::ALL))
        .x_axis(Axis::default().title("t").style(Style::default().fg(Color::Gray)).labels(x_labels).bounds([0., 100.]))//BALISE: N0
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0%".bold(), "50%".bold(), "100%".bold()])
                .bounds([0., 100.]),
        );

    frame.render_widget(chart, area);
}

fn render_memory(frame: &mut Frame, app: &App, area: Rect) {
    let x_labels = vec![];

    let fserie: Vec<f64> = app.data.memory_usage.clone().into_iter().map(|elm| elm as f64).collect();
    let serie = smooth_serie(&fserie, 7);

    let datasets = vec![
        Dataset::default()
            .name("RAM")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Magenta))
            .data(&serie),
    ];

    let chart = Chart::new(datasets)
        .block(Block::default().title("Memory".magenta().bold()).borders(Borders::ALL))
        .x_axis(Axis::default().title("t").style(Style::default().fg(Color::Gray)).labels(x_labels).bounds([0., 100.]))//BALISE: N0
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0%".bold(), "50%".bold(), "100%".bold()])
                .bounds([0., app.data.total_memory as f64]),
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
