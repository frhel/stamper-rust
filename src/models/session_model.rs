use chrono::Utc;

use stamper::db_controller;

use crate::models::song_model::CurrentSong;
// import db_controller from the root of the project

#[derive(Debug, Clone)]
pub struct SessionModel {
    _start_time: String,
    _yt_id: String,
    _songs: Vec<CurrentSong>
}

impl Default for SessionModel {
    fn default() -> Self {
        SessionModel {
            _start_time: Utc::now().to_string(),
            _yt_id: String::new(),
            _songs: Vec::new()
        }
    }
}

pub fn init_session() -> SessionModel {
    // Initialize the database connection just in case.
    db_controller::init_db().expect("Failed to initialize database");

    let session = SessionModel { ..Default::default() };

    // Check if there is an active session in the database
    // If there is, load it into the session model
    // If there isn't, create a new session in the database
    // and load it into the session model
    let _conn = db_controller::open_db_connection();

    // Check if there is an active session in the database




    session
}



