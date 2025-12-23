use ratatui::terminal::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::song::Song;

pub fn draw_ui(
    f: &mut Frame,
    area: Rect,
    songs: &Vec<Song>,
    list_state: &ListState,
    current_play_idx: Option<usize>,
    playing: bool,
    volume: i64,
    repeat: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(area);

    // Now playing
    let top_block = Block::default().borders(Borders::ALL).title(" Now Playing ");
    let now_text: Vec<Line> = if let Some(idx) = current_play_idx {
        let s = &songs[idx];
        vec![
            Line::from(Span::raw(format!("{} - {}", s.artist, s.title))),
            Line::from(Span::raw(format!(
                "Status: {}   Vol: {}%   Repeat: {}",
                if playing { "Playing" } else { "Paused" },
                volume,
                if repeat { "On" } else { "Off" }
            ))),
        ]
    } else {
        vec![Line::from(Span::raw("Nothing playing"))]
    };
    let paragraph = Paragraph::new(now_text).block(top_block);
    f.render_widget(paragraph, chunks[0]);

    // Playlist + album art
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
        .split(chunks[1]);

    // Playlist
    let items: Vec<ListItem> = songs
        .iter()
        .map(|s| ListItem::new(format!("{} - {}", s.artist, s.title)))
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Playlist "))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, bottom[0], &mut list_state.clone());

    // Album art info (press 'o' to open actual image)
    let art_block = Block::default().borders(Borders::ALL).title(" Album Art ");
    let mut text = vec![];
    if let Some(idx) = current_play_idx {
        let s = &songs[idx];
        text.push(Line::from(Span::raw(format!("Title: {}", s.title))));
        text.push(Line::from(Span::raw(format!("Artist: {}", s.artist))));
        if let Some(ref art) = s.album_art_path {
            text.push(Line::from(Span::raw(format!("Cover: {}", art))));
            text.push(Line::from(Span::raw("Press 'o' to open cover image")));
        } else {
            text.push(Line::from(Span::raw("No embedded cover found")));
        }
    } else {
        text.push(Line::from(Span::raw("No song selected")));
    }
    let para = Paragraph::new(text).block(art_block);
    f.render_widget(para, bottom[1]);
}
