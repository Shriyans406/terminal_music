// src/main.rs
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use directories::ProjectDirs;
use id3::Tag;
use id3::TagLike;
use named_pipe::PipeClient;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use serde_json::json;
use walkdir::WalkDir;


//use crossterm::event::{Event as CEvent, KeyCode, KeyEvent, KeyEventKind};

#[derive(Clone)]
struct Song {
    path: String,
    title: String,
    artist: String,
    album_art_path: Option<String>,
}

enum Event<I> {
    Input(I),
    Tick,
}

fn extract_cover(tag: &Tag, out_dir: &Path, base_name: &str) -> Option<String> {
    if let Some(picture) = tag.pictures().next() {
        let ext = match picture.mime_type.as_str() {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            _ => "bin",
        };
        let filename = format!("{}.{}", base_name, ext);
        let outfile = out_dir.join(&filename);
        if let Ok(mut f) = File::create(&outfile) {
            if f.write_all(&picture.data).is_ok() {
                return Some(outfile.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn scan_music(music_dir: &Path, cover_out: &Path) -> Vec<Song> {
    let mut songs = Vec::new();
    if !music_dir.exists() {
        eprintln!("Music folder not found: {}", music_dir.display());
        return songs;
    }

    for entry in WalkDir::new(music_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| {
                    let ext = ext.to_lowercase();
                    matches!(ext.as_str(), "mp3" | "flac" | "wav" | "m4a")
                })
                .unwrap_or(false)
        })
    {
        let path = entry.path().to_path_buf();
        let path_str = path.display().to_string();
        let tag = Tag::read_from_path(&path).ok();

        let title = tag
            .as_ref()
            .and_then(|t| t.title())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown Title")
                    .to_string()
            });

        let artist = tag
            .as_ref()
            .and_then(|t| t.artist())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let base_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("cover");

        let album_art_path = tag
            .as_ref()
            .and_then(|t| extract_cover(t, cover_out, base_name));

        songs.push(Song {
            path: path_str,
            title,
            artist,
            album_art_path,
        });
    }

    songs
}

fn spawn_mpv_with_pipe(music_pipe: &str) -> io::Result<std::process::Child> {
    // start mpv in idle mode with IPC server
    let child = Command::new("mpv")
        .arg("--no-video")
        .arg("--idle=yes")
        .arg(format!("--input-ipc-server={}", music_pipe))
        .spawn()?;
    Ok(child)
}

fn connect_pipe_with_retry(pipe_name: &str, tries: usize, delay_ms: u64) -> io::Result<PipeClient> {
    for _ in 0..tries {
        match PipeClient::connect(pipe_name) {
            Ok(c) => return Ok(c),
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(delay_ms)),
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, "Failed to connect pipe"))
}

fn send_json_command(pipe: &mut PipeClient, cmd: serde_json::Value) -> io::Result<()> {
    let s = serde_json::to_vec(&cmd)?;
    pipe.write_all(&s)?;
    pipe.write_all(b"\n")?;
    Ok(())
}

fn open_with_default(path: &str) {
    // Use cmd start to open with default application
    let _ = Command::new("cmd")
        .args(["/C", "start", "", path])
        .spawn();
}

fn ui(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    songs: &Vec<Song>,
    list_state: &ListState,
    current_play_idx: Option<usize>,
    playing: bool,
    volume: i64,
    repeat: bool,
) {
    // layout: top (now playing) + bottom (left playlist, right album art)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(area);

    // Top: now playing
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
        vec![Line::from(Span::raw("Nothing playing")), Line::from(Span::raw(""))]
    };
    let paragraph = Paragraph::new(now_text).block(top_block);
    f.render_widget(paragraph, chunks[0]);

    // Bottom split
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
        .split(chunks[1]);

    // Playlist (left)
    let items: Vec<ListItem> = songs
        .iter()
        .map(|s| {
            let line = format!("{} - {}", s.artist, s.title);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Playlist "))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, bottom[0], &mut list_state.clone());

    // Right: album art + metadata
    let art_block = Block::default().borders(Borders::ALL).title(" Album Art ");
    let mut text = vec![];
    if let Some(idx) = current_play_idx {
        let s = &songs[idx];
        text.push(Line::from(Span::raw(format!("Title: {}", s.title))));
        text.push(Line::from(Span::raw(format!("Artist: {}", s.artist))));
        text.push(Line::from(Span::raw("")));
        if let Some(ref art) = s.album_art_path {
            text.push(Line::from(Span::styled(
                format!("Cover: {}", art),
                Style::default().add_modifier(Modifier::BOLD),
            )));
            text.push(Line::from(Span::raw("Press 'o' to open cover image.")));
        } else {
            text.push(Line::from(Span::raw("No embedded cover found.")));
        }
    } else {
        text.push(Line::from(Span::raw("No song selected.")));
    }
    let para = Paragraph::new(text).block(art_block);
    f.render_widget(para, bottom[1]);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // project dirs for cover cache
    let proj_dirs = ProjectDirs::from("com", "terminal", "music-cli")
        .ok_or("Cannot find project dir (ProjectDirs)")?;
    let cache_dir = proj_dirs.cache_dir();
    fs::create_dir_all(cache_dir)?;
    let cover_dir = cache_dir.join("covers");
    fs::create_dir_all(&cover_dir)?;

    // explicit music directory (change if needed)
    let music_dir = Path::new(r"C:\Users\HP\Music");

    println!("Scanning {}", music_dir.display());
    let mut songs = scan_music(&music_dir, &cover_dir);
    if songs.is_empty() {
        println!("No audio files found in {}", music_dir.display());
        return Ok(());
    }
    songs.sort_by(|a, b| a.artist.cmp(&b.artist).then(a.title.cmp(&b.title)));

    // mpv pipe & spawn
    let pipe_name = r"\\.\pipe\mpvpipe";
    let _mpv_child = spawn_mpv_with_pipe(pipe_name)
        .map_err(|e| format!("Failed to spawn mpv: {}. Ensure mpv is installed in PATH.", e))?;

    let mut pipe = connect_pipe_with_retry(pipe_name, 20, 100)
        .map_err(|_| "Failed to connect to mpv IPC pipe. Make sure mpv is running with --input-ipc-server.")?;

    // Setup terminal + input thread
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let (tx, rx) = std::sync::mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    std::thread::spawn(move || loop {
        if event::poll(tick_rate).unwrap() {
            if let CEvent::Key(key) = event::read().unwrap() {
                let _ = tx.send(Event::Input(key));
            }
        } else {
            let _ = tx.send(Event::Tick);
        }
    });

    // UI state
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut list_idx: usize = 0;
    let mut current_play_idx: Option<usize> = None;
    let mut playing = false;
    let mut volume: i64 = 50;
    let mut repeat = false;

    // set initial volume in mpv
    send_json_command(&mut pipe, json!({"command": ["set_property", "volume", volume]}))?;

    // main loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            ui(
                f,
                size,
                &songs,
                &list_state,
                current_play_idx,
                playing,
                volume,
                repeat,
            );
        })?;

        match rx.recv()? {
            Event::Input(event) => match event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                } => break,
                KeyEvent {
                    code: KeyCode::Down,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    if list_idx + 1 < songs.len() {
                        list_idx += 1;
                        list_state.select(Some(list_idx));
                    }
                }
                KeyEvent {
                    code: KeyCode::Up,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    if list_idx > 0 {
                        list_idx -= 1;
                        list_state.select(Some(list_idx));
                    }
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    let idx = list_idx;
                    let song = &songs[idx];
                    // loadfile
                    send_json_command(&mut pipe, json!({"command": ["loadfile", song.path]}))?;
                    current_play_idx = Some(idx);
                    playing = true;
                    send_json_command(&mut pipe, json!({"command": ["set_property", "volume", volume]}))?;
                }
                KeyEvent {
                    code: KeyCode::Char(' '),
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    send_json_command(&mut pipe, json!({"command": ["cycle", "pause"]}))?;
                    playing = !playing;
                }
                KeyEvent {
                    code: KeyCode::Char('n'),
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    if let Some(idx) = current_play_idx {
                        let next_idx = (idx + 1) % songs.len();
                        let song = &songs[next_idx];
                        send_json_command(&mut pipe, json!({"command": ["loadfile", song.path]}))?;
                        current_play_idx = Some(next_idx);
                        playing = true;
                        list_idx = next_idx;
                        list_state.select(Some(list_idx));
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('p'),
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    if let Some(idx) = current_play_idx {
                        let prev_idx = if idx == 0 { songs.len() - 1 } else { idx - 1 };
                        let song = &songs[prev_idx];
                        send_json_command(&mut pipe, json!({"command": ["loadfile", song.path]}))?;
                        current_play_idx = Some(prev_idx);
                        playing = true;
                        list_idx = prev_idx;
                        list_state.select(Some(list_idx));
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('s'),
                    //kind: KeyEventKind::Press,
                    ..
                } => {
                    send_json_command(&mut pipe, json!({"command": ["stop"]}))?;
                    playing = false;
                    current_play_idx = None;
                }
                KeyEvent {
                    code: KeyCode::Char('+'),
                    //kind: KeyEventKind::Press,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('='),
                    ..
                } => {
                    if volume < 200 {
                        volume += 5;
                        send_json_command(&mut pipe, json!({"command": ["set_property", "volume", volume]}))?;
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('-'),
                    //kind: KeyEventKind::Press,
                    ..
                } => {
                    if volume > 0 {
                        volume -= 5;
                        send_json_command(&mut pipe, json!({"command": ["set_property", "volume", volume]}))?;
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('r'),
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    repeat = !repeat;
                    send_json_command(&mut pipe, json!({"command": ["set_property", "loop-playlist", repeat]}))?;
                }
                KeyEvent {
                    code: KeyCode::Char('o'),
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    if let Some(idx) = current_play_idx {
                        if let Some(ref art) = songs[idx].album_art_path {
                            open_with_default(art);
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {
                // future: poll time position, update progress bar, etc.
            }
        }
    }

    // cleanup terminal
    disable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
