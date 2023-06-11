use chrono::{Utc, DateTime};
use rusqlite::Connection;

use crate::models::song_model::CurrentSong;
// import db_controller from the root of the project

#[derive(Debug, Clone)]
pub struct SessionModel {
    pub start_time: DateTime<Utc>,
    pub yt_id: String,
    pub songs: Vec<CurrentSong>
}

impl Default for SessionModel {
    fn default() -> Self {
        SessionModel {
            start_time: Utc::now(),
            yt_id: String::new(),
            songs: Vec::new()
        }
    }
}

pub fn load_active_session(conn: Connection, session: SessionModel, newest_filestamp: DateTime<Utc>) -> SessionModel {
    // Here we take the timestamp of the most recently created video
    // and use it to determine the active session
    session
}


