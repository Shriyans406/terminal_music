use crate::song::Song;
use serde::Deserialize;
use reqwest::blocking::Client;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct JamendoResponse {
    results: Vec<JamendoTrack>,
}

#[derive(Debug, Deserialize)]
struct JamendoTrack {
    name: String,
    artist_name: String,
    audio: String,
}

pub fn search_online_songs(query: &str) -> Result<Vec<Song>, Box<dyn Error>> {
    let client_id = "23d25192"; // keep it here, internal
    let url = format!(
        "https://api.jamendo.com/v3.0/tracks/?client_id={}&format=json&limit=10&namesearch={}",
        client_id, query
    );

    let client = Client::new();
    let response: JamendoResponse = client.get(url).send()?.json()?;

    let songs = response
        .results
        .into_iter()
        .map(|t| Song {
            title: t.name,
            artist: t.artist_name,
            path: t.audio, // streaming URL
            is_online: true,
            album_art_path: None,
        })
        .collect();

    Ok(songs)
}
