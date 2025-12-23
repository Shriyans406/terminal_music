mod app;
mod song;
mod music;
mod mpv;
mod ui;
mod utils;


use crate::app::AppState;
//use crate::song::Song;
use crate::music::scan_music;
//use crate::mpv::{connect_pipe_with_retry, spawn_mpv_with_pipe};
use crate::mpv::connect_pipe_with_retry;

use ratatui::widgets::ListState;





fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Project dirs & music scan
    let music_dir = std::path::Path::new(r"C:\Users\HP\Music");
    let cover_dir = std::path::Path::new(r"C:\Users\HP\.cache\music-cli\covers");

    let songs = scan_music(music_dir, cover_dir);

    // Pipe name
    let pipe_name = r"\\.\pipe\mpvpipe";

    // Connect to mpv
    let mut pipe = connect_pipe_with_retry(pipe_name, 50, 100)?;

    let app = AppState {
        songs,
        list_state: ListState::default(),
        current_play_idx: None,
        playing: false,
        volume: 50,
        repeat: false,
    };
    app::run(&mut pipe, pipe_name, app)?;
    Ok(())
}
