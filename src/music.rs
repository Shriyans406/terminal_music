use crate::song::Song;
use id3::{Tag, TagLike};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

pub fn extract_cover(tag: &Tag, out_dir: &Path, base_name: &str) -> Option<String> {
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

pub fn scan_music(music_dir: &Path, cover_out: &Path) -> Vec<Song> {
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
        let tag = Tag::read_from_path(&path).ok();

        let title = tag
    .as_ref()
    .and_then(|t| t.title())
    .unwrap_or("Unknown Title")
    .to_string();

let artist = tag
    .as_ref()
    .and_then(|t| t.artist())
    .unwrap_or("Unknown Artist")
    .to_string();


        let base_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("cover");

        let album_art_path = tag.as_ref().and_then(|t| extract_cover(t, cover_out, base_name));

        songs.push(Song {
            path: path.display().to_string(),
            title,
            artist,
            album_art_path,
            is_online: false,
        });
    }

    songs
}
