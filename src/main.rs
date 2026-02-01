mod app;
mod song;
mod music;
mod mpv;
mod ui;
mod utils;
mod online;

use crate::app::AppState;
//use crate::song::Song;
use crate::music::scan_music;
use crate::mpv::{connect_pipe_with_retry, spawn_mpv_with_pipe};
//use crate::mpv::connect_pipe_with_retry;


//use crate::online::search_online_songs;



use ratatui::widgets::ListState;





fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Project dirs & music scan
    let music_dir = std::path::Path::new(r"C:\Users\HP\Music");
    let cover_dir = std::path::Path::new(r"C:\Users\HP\.cache\music-cli\covers");

    let songs = scan_music(music_dir, cover_dir);

    // Pipe name
    let pipe_name = r"\\.\pipe\mpvpipe";

 // START MPV FIRST (THIS WAS MISSING)
    let _mpv = spawn_mpv_with_pipe(pipe_name)?;

    // Give mpv a moment to create the pipe
    std::thread::sleep(std::time::Duration::from_millis(300));


    //let online = search_online_songs("love", "23d25192")?;
//println!("{:#?}", online);
//return Ok(());



    // Connect to mpv
    let mut pipe = connect_pipe_with_retry(pipe_name, 50, 100)?;

    let app = AppState {
        songs,
        list_state: ListState::default(),
        current_play_idx: None,
        playing: false,
        volume: 50,
        repeat: false,

         // 🔽 Phase 3 additions
    search_mode: false,
    search_query: String::new(),
    };
    app::run(&mut pipe, pipe_name, app)?;
    Ok(())
}
