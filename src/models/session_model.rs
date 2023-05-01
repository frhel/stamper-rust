use crate::models::song_model::CurrentSong;
use tokio::time::*;

#[derive(Debug, Clone)]
pub struct SessionModel {
    start_time: Instant,
    yt_id: String,
    songs: Vec<CurrentSong>
}

impl Default for SessionModel {
    fn default() -> Self {
        SessionModel {
            start_time: Instant::now().into(),
            yt_id: String::new(),
            songs: Vec::new()
        }
    }
}

pub fn init_session() -> SessionModel {
    let session = SessionModel { ..Default::default() };
    println!("Session: {:?}", session);
    session
}