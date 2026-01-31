use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct OnlineSong {
    pub title: String,
    pub artist: String,
    pub stream_url: String,
}

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

pub fn search_online_songs(
    query: &str,
    client_id: &str,
) -> Result<Vec<OnlineSong>, Box<dyn Error>> {
    let url = format!(
        "https://api.jamendo.com/v3.0/tracks/?client_id={}&format=json&limit=10&namesearch={}",
        client_id, query
    );

    let response: JamendoResponse = reqwest::blocking::get(&url)?.json()?;

    let songs = response
        .results
        .into_iter()
        .map(|track| OnlineSong {
            title: track.name,
            artist: track.artist_name,
            stream_url: track.audio,
        })
        .collect();

    Ok(songs)
}
