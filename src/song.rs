#[derive(Clone, Debug)]
pub struct Song {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album_art_path: Option<String>,
    pub is_online: bool,
}
