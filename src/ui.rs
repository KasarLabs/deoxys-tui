use std::cmp::min;
use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use humansize::{format_size, BINARY};
use ratatui::layout::{Constraint, Direction, Layout, Margin, Offset, Rect};
use ratatui::prelude::Frame;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, Paragraph};
use starknet::core::types::SyncStatusType;

use crate::app::App;

pub fn ui(app: &App, frame: &mut Frame) {
    let node0 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.size());

    render_zone(frame, node0[1], "System");

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50); 2])
        .split(node0[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(33); 3])
        .split(node0[1].inner(&Margin::new(2, 1)));

    let cpu_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
        .split(right[0]);
    let memory_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
        .split(right[1]);
    let storage_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(right[2].inner(&Margin::new(1, 1)));

    render_cpu_gauge(frame, app, cpu_area[1].inner(&Margin::new(1, 0)));
    render_cpu_graph(frame, app, cpu_area[0]);

    render_memory_gauge(frame, app, memory_area[1].inner(&Margin::new(1, 0)));
    render_memory_graph(frame, app, memory_area[0]);

    render_zone(frame, right[2], "Storage");
    render_storage_data(frame, app, storage_area[0]);
    render_storage_gauge(frame, app, storage_area[1]);

    render_l1_logs(frame, app, left[0]);
    render_l2_logs(frame, app, left[1]);
}

fn _render_sync(frame: &mut Frame, app: &App, area: Rect) {
    let text = match app.data.syncing.clone() {
        Ok(SyncStatusType::Syncing(status)) => format!(
            "Starting: {} Current: {} Highest: {}",
            status.starting_block_num, status.current_block_num, status.highest_block_num
        ),
        Ok(SyncStatusType::NotSyncing) => "Not Syncing".to_string(),
        Err(err) => err.clone(),
    };
    frame.render_widget(Block::new().title("Syncing").borders(Borders::ALL), area);
    frame.render_widget(Paragraph::new(text.as_str()).light_green(), area.inner(&Margin::new(2, 1)));
}

fn smooth_serie(series: &[f64], window_size: usize) -> Vec<(f64, f64)> {
    let mut smoothed_series = Vec::new();

    let ignore_count = window_size / 2;

    for i in ignore_count..series.len() - ignore_count {
        let window_average: f64 = series[i - ignore_count..=i + ignore_count].iter().sum::<f64>() / window_size as f64;
        smoothed_series.push(window_average);
    }
    let x: Vec<f64> = (0..=100).map(|x| x as f64).collect();
    let serie: Vec<(f64, f64)> = x.into_iter().zip(smoothed_series).collect();
    serie
}

fn render_zone(frame: &mut Frame, area: Rect, title: &str) {
    let outline = Block::new().borders(Borders::ALL).title(title);
    frame.render_widget(outline, area);
}

fn render_storage_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let ratio = app.data.disk_usage as f64 / app.data.disk_size as f64;
    render_zone(frame, area, "Used");
    render_gauge(frame, area.inner(&Margin::new(1, 1)), (ratio * 100.) as u16, true);
}

fn render_l1_logs(frame: &mut Frame, _app: &App, area: Rect) {
    render_zone(frame, area, "L1 logs")
}

fn render_l2_logs(frame: &mut Frame, _app: &App, area: Rect) {
    render_zone(frame, area, "L2 logs");
}

fn _render_rpc_playground(frame: &mut Frame, _app: &App, area: Rect) {
    render_zone(frame, area, "RPC Playground")
}

fn render_cpu_graph(frame: &mut Frame, app: &App, area: Rect) {
    let serie = smooth_serie(&app.data.cpu_usage, 10);
    let datasets = vec![
        Dataset::default()
            .name("CPU")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(&serie),
    ];
    let chart = Chart::new(datasets)
        .block(Block::default().title("CPU".cyan().bold()).borders(Borders::ALL))
        .x_axis(Axis::default().title("t").style(Style::default().fg(Color::Gray)).labels(vec![]).bounds([0., 100.]))//BALISE: N0
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0%".bold(), "50%".bold(), "100%".bold()])
                .bounds([0., 101.]),
        );
    frame.render_widget(chart, area);
}

fn render_cpu_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let serie = smooth_serie(&app.data.cpu_usage, 20);
    render_gauge(frame, area, serie.last().unwrap().1 as u16, true)
}

fn render_memory_gauge(frame: &mut Frame, app: &App, area: Rect) {
    render_gauge(
        frame,
        area,
        ((*app.data.memory_usage.last().unwrap() as f64 / app.data.total_memory as f64) * 100.) as u16,
        true,
    );
}

#[inline]
fn render_gauge(frame: &mut Frame, area: Rect, percent: u16, alert_mode: bool) {
    let color;

    if alert_mode {
        if percent <= 100 / 3 {
            color = Color::Green;
        } else if percent <= 200 / 3 {
            color = Color::Rgb(255, 128, 0)
        } else {
            color = Color::Red;
        }
    } else {
        color = Color::Green
    }
    let gauge = Gauge::default().gauge_style(color).bg(Color::Rgb(20, 20, 20)).percent(min(percent, 100));
    frame.render_widget(gauge, area);
}

fn render_memory_graph(frame: &mut Frame, app: &App, area: Rect) {
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
        .x_axis(Axis::default().title("t").style(Style::default().fg(Color::Gray)).labels(vec![]).bounds([0., 100.]))//BALISE: N0
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0%".bold(), "50%".bold(), "100%".bold()])
                .bounds([0., app.data.total_memory as f64]),
        );
    frame.render_widget(chart, area);
}

fn render_storage_data(frame: &mut Frame, app: &App, area: Rect) {
    let total_space =
        Line::raw(format!("Total Disk Space: {}", format_size(app.data.disk_size, BINARY))).style(Color::Green);
    let used_space =
        Line::raw(format!("Node Disk Usage: {}", format_size(app.data.disk_usage, BINARY))).style(Color::Green);
    let available_space =
        Line::raw(format!("Available Space: {}", format_size(app.data.available_storage, BINARY))).style(Color::Green);

    let zones =
        Layout::default().direction(Direction::Vertical).constraints(vec![Constraint::Percentage(33); 3]).split(area);

    frame.render_widget(total_space, zones[0]);
    frame.render_widget(used_space, zones[0].offset(Offset { x: 0, y: 1 }));
    frame.render_widget(available_space, zones[0].offset(Offset { x: 0, y: 2 }));
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
