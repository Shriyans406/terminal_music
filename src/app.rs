use std::{
    io::stdout,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

//use named_pipe::PipeClient;


use crate::{
    song::Song,
    ui::draw_ui,
};

use ratatui::widgets::ListState;

use crate::mpv::{PipeClient, send_json_command};
use crate::utils::open_with_default;

pub struct AppState {
    pub songs: Vec<Song>,
    pub list_state: ListState,
    pub current_play_idx: Option<usize>,
    pub playing: bool,
    pub volume: i64,
    pub repeat: bool,
}

pub fn run(
    pipe: &mut PipeClient,
    _pipe_name: &str,
    mut app: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
    use crossterm::execute;
    use std::io::stdout;
    use crate::ui::draw_ui;

    enable_raw_mode()?;
    execute!(stdout(), crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    use std::time::Instant;
    let mut last_input = Instant::now();


    loop {
        terminal.draw(|f| {
            let size = f.size();
            draw_ui(
                f,
                size,
                &app.songs,
                &app.list_state,
                app.current_play_idx,
                app.playing,
                app.volume,
                app.repeat,
            );
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if last_input.elapsed() < Duration::from_millis(150) {
    continue;
}
last_input = Instant::now();


                match key.code {


                    
                    KeyCode::Char('q') => break,

                    KeyCode::Down => {
                        let i = app.list_state.selected().unwrap_or(0);
                        if i + 1 < app.songs.len() {
                            app.list_state.select(Some(i + 1));
                        }
                    }

                    KeyCode::Up => {
                        let i = app.list_state.selected().unwrap_or(0);
                        if i > 0 {
                            app.list_state.select(Some(i - 1));
                        }
                    }

                    KeyCode::Enter => {
                        if let Some(i) = app.list_state.selected() {
                            let path = app.songs[i].path.clone();
                            send_json_command(
                                pipe,
                                _pipe_name,
                                serde_json::json!({
                                    "command": ["loadfile", path, "replace"]
                                }),
                            )?;
                            app.current_play_idx = Some(i);
                            app.playing = true;
                        }
                    }


                    KeyCode::Char(' ') => {
    send_json_command(
        pipe,
        _pipe_name,
        serde_json::json!({
            "command": ["cycle", "pause"]
        }),
    )?;
    app.playing = !app.playing;
}







                    KeyCode::Char('+') => {
                        send_json_command(
                            pipe,
                            _pipe_name,
                            serde_json::json!({"command": ["add", "volume", 5]}),
                        )?;
                        app.volume += 5;
                    }

                    KeyCode::Char('-') => {
                        send_json_command(
                            pipe,
                            _pipe_name,
                            serde_json::json!({"command": ["add", "volume", -5]}),
                        )?;
                        app.volume -= 5;
                    }


                    KeyCode::Char('r') => {
    app.repeat = !app.repeat;
    let loop_cmd = if app.repeat {
        serde_json::json!({"command": ["set_property", "loop-file", "inf"]})
    } else {
        serde_json::json!({"command": ["set_property", "loop-file", "no"]})
    };
    send_json_command(pipe, _pipe_name, loop_cmd)?;
}






                    KeyCode::Char('o') => {
                        if let Some(i) = app.current_play_idx {
                            if let Some(ref art) = app.songs[i].album_art_path {
                                open_with_default(art);
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

