use chrono::{Utc, NaiveDateTime, DateTime};

use stamper::db_controller;

use crate::models::{SessionModel, load_active_session};

pub fn init_session() -> SessionModel {
    println!("Initializing session...");
    // Initialize the database connection just in case.
    db_controller::init_db().expect("Failed to initialize database");
    let conn = db_controller::open_db_connection();

    let mut session = SessionModel { ..Default::default() };

    // Get the creation date of the latest video file in the VOD directory
    // and use it to determine the active session
    let newest_filestamp = get_latest_video_timestamp();

    // Check if there is an active session in the database
    // If there is, load it into the session model
    // If there isn't, create a new session in the database
    // and load it into the session model
    session = load_active_session(conn, session, newest_filestamp);


    session
}

fn get_latest_video_timestamp() -> DateTime<Utc> {
    let vod_dir = "E:\\VODs\\Temp";
    println!("VOD directory: {}", vod_dir);

    
    // Find the newest file of type .mkv in the VOD directory
    let latest_video = std::fs::read_dir(vod_dir)
        .expect("Failed to read VOD directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter(|entry| entry.path().extension().unwrap() == "mkv")
        .max_by_key(|entry| entry.path())
        .expect("Failed to find a video file in the VOD directory");

    let timestamp = latest_video.metadata()
        .expect("Failed to get metadata for latest video")
        .created()
        .expect("Failed to get creation date for latest video")
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    // Convert the timestamp to a Chrono DateTime
    let stamp: DateTime<Utc> = DateTime::from_utc(
        NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap(), Utc
    );
    
    stamp
}