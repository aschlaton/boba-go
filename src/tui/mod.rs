use std::io;

#[derive(Copy, Clone)]
pub enum StartAction {
    NewLocalGame,
    HowToPlay,
    Quit,
}

const LEFT_ASCII: &str = include_str!("assets/title.txt");
const RIGHT_ASCII: &str = include_str!("assets/boba_art.txt");

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

pub fn run_start_page() -> StartAction {
    // Setup terminal
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let mut selected: usize = 0;
    let options = [
        ("Start new local game", StartAction::NewLocalGame),
        ("How to play", StartAction::HowToPlay),
        ("Quit", StartAction::Quit),
    ];

    let result = loop {
        let _ = terminal.draw(|f| {
            let area = f.area();
            // Outer frame around everything
            let outer = Block::default()
                .title(Span::styled(
                    "Main Menu",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            f.render_widget(outer.clone(), area);
            let size = outer.inner(area);

            // Dynamic title height: leave room for menu+footer
            let title_lines = LEFT_ASCII.lines().count().max(RIGHT_ASCII.lines().count()) as u16;
            let max_title = size.height.saturating_sub(4);
            let title_height = title_lines.min(max_title);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(title_height),
                    Constraint::Min(3),
                    Constraint::Length(2),
                ])
                .split(size);

            // Title row: split into left/right with dynamic widths so content isn't cut
            let left_w: u16 = LEFT_ASCII.lines().map(|l| l.chars().count() as u16).max().unwrap_or(30);
            let right_w: u16 = RIGHT_ASCII.lines().map(|l| l.chars().count() as u16).max().unwrap_or(30);
            let total_w = left_w.saturating_add(right_w).max(1);
            // compute percentages; ensure at least 10%/10%
            let left_pct = ((left_w as u32 * 100) / (total_w as u32)).clamp(10, 90) as u16;
            let right_pct = 100u16.saturating_sub(left_pct);
            let title_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(left_pct),
                    Constraint::Percentage(right_pct),
                ])
                .split(chunks[0]);

            // Bottom-justify the shorter art by padding with leading newlines
            let left_lines_ct = LEFT_ASCII.lines().count();
            let right_lines_ct = RIGHT_ASCII.lines().count();
            let pad_lines = right_lines_ct.saturating_sub(left_lines_ct);
            let padded_left = if pad_lines > 0 {
                let mut s = String::new();
                for _ in 0..pad_lines { s.push('\n'); }
                s.push_str(LEFT_ASCII);
                s
            } else {
                LEFT_ASCII.to_string()
            };

            let left = Paragraph::new(Text::from(padded_left))
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Left)
                .block(Block::default());
            f.render_widget(left, title_row[0]);

            // Colorize boba art with brown-sugar streaks on a mostly white cup
            let streak_dark = Color::Rgb(90, 56, 38);
            let streak_med = Color::Rgb(153, 100, 60);
            let streak_light = Color::Rgb(210, 160, 100);
            let cup_white = Color::White;

            let right_lines_raw: Vec<&str> = RIGHT_ASCII.lines().collect();
            let total_lines = right_lines_raw.len().max(1);

            // Detect where the drink (solid fill) actually starts: first line with enough solid blocks
            let is_solid = |c: char| matches!(c, '⣿' | '⣷' | '⣾');
            let mut drink_top_row = total_lines / 2; // fallback
            for (i, ln) in right_lines_raw.iter().enumerate() {
                let solid_count = ln.chars().filter(|&c| is_solid(c)).count();
                if solid_count >= 5 { // threshold heuristic
                    drink_top_row = i;
                    break;
                }
            }

            let right_lines: Vec<Line> = right_lines_raw
                .iter()
                .enumerate()
                .map(|(row, ln)| {
                    let mut spans: Vec<Span> = Vec::with_capacity(ln.chars().count());
                    for (col, ch) in ln.chars().enumerate() {
                        // Only color interior "solid" cup fill glyphs; leave borders/other glyphs uncolored
                        if is_solid(ch) {
                            let color = if row < drink_top_row {
                                // Above drink: mostly white with very faint drips
                                let h = (row as u32).wrapping_mul(13) ^ (col as u32).wrapping_mul(17);
                                if h % 47 == 0 { streak_light } else { cup_white }
                            } else {
                                // Within drink: cloud-like blobs using simple 2D hash-based density
                                let drink_height = total_lines.saturating_sub(drink_top_row).max(1);
                                let max_depth = (drink_height as f32 * 0.38).ceil() as usize; // only top ~38% of drink has syrup
                                let depth = row.saturating_sub(drink_top_row);
                                if depth > max_depth { cup_white } else {
                                    // Accumulate pseudo-noise from a few neighborhood samples to form blobs
                                    let p = |r: i32, c: i32| -> u32 {
                                        let rr = (r.max(0) as u32).wrapping_mul(1103515245);
                                        let cc = (c.max(0) as u32).wrapping_mul(12345);
                                        rr ^ cc ^ 0x9E3779B9
                                    };
                                    let r = row as i32;
                                    let c = col as i32;
                                    let n = p(r, c)
                                        .wrapping_add(p(r-1, c+2))
                                        .wrapping_add(p(r+1, c-1))
                                        .wrapping_add(p(r-2, c))
                                        .wrapping_add(p(r, c+3));
                                    // Normalize to [0, 99]
                                    let d = (n % 100) as u32;
                                    // More density near the drink top, fading with depth
                                    let fade = 100u32.saturating_sub(((depth * 100) / (max_depth.max(1))).min(100) as u32);
                                    let density = d.saturating_add(fade / 3);
                                    if density > 140 {
                                        streak_dark
                                    } else if density > 110 {
                                        streak_med
                                    } else if density > 90 {
                                        streak_light
                                    } else {
                                        cup_white
                                    }
                                }
                            };
                            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                        } else {
                            // Preserve other chars and spacing as-is
                            spans.push(Span::raw(ch.to_string()));
                        }
                    }
                    Line::from(spans)
                })
                .collect();
            let right = Paragraph::new(right_lines)
                .alignment(Alignment::Left)
                .block(Block::default());
            f.render_widget(right, title_row[1]);

            // Menu
            let items: Vec<ListItem> = options
                .iter()
                .enumerate()
                .map(|(i, (label, _))| {
                    let style = if i == selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Black)
                    };
                    // Two-line item: label + spacer line for extra vertical padding
                    ListItem::new(vec![
                        Line::from(Span::styled(*label, style)),
                        Line::from("")
                    ])
                })
                .collect();
            let list = List::new(items)
                .block(Block::default())
                .highlight_style(Style::default().bg(Color::Rgb(180, 180, 180)))
                .highlight_symbol("▶ ");
            f.render_widget(list, chunks[1]);

            // Footer
            let footer = Paragraph::new(Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" to navigate  "),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" to select  "),
                Span::styled("Q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" to quit"),
            ]))
            .alignment(Alignment::Center)
            .block(Block::default());
            f.render_widget(footer, chunks[2]);
        });

        if let Ok(true) = event::poll(std::time::Duration::from_millis(200)) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break StartAction::Quit,
                        KeyCode::Up => {
                            if selected == 0 { selected = options.len() - 1; } else { selected -= 1; }
                        }
                        KeyCode::Down => {
                            selected = (selected + 1) % options.len();
                        }
                        KeyCode::Enter => break options[selected].1.clone(),
                        _ => {}
                    }
                }
            }
        }
    };

    // Restore terminal
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    result
}


