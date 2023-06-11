use rusqlite::Connection;

// ----------------------------------------------------------------------------
// Define the database configuration struct
struct DbConfig {
    db_name: String,
}

// ----------------------------------------------------------------------------
// Set the default DbConfig db_name to the database name to use
impl Default for DbConfig {
    fn default() -> Self {
        DbConfig {
            db_name: "stamper_data".to_string(),
        }
    }
}

// ----------------------------------------------------------------------------
// Open database connection and return the connection
pub fn open_db_connection() -> Connection {
    let db_name = DbConfig::default().db_name;

    return Connection::open(&db_name).expect("Failed to open database connection");
}

// ----------------------------------------------------------------------------
// Close database connection
pub fn close_db_connection(conn: Connection) {
    conn.close().expect("Failed to close database connection");
}

// ----------------------------------------------------------------------------
// The database initialization function that's called on app startup
pub fn init_db() -> Result<(), String> {
    let db_name = DbConfig::default().db_name;

    println!("Initializing database connection...");

    // Check if the database file exists
    if std::path::Path::new(&db_name).exists() {
        // break early if the database file exists
        println!("Database initialized");
        return Ok(());
    }

    // Create the database file if it doesn't exist
    let conn = Connection::open(&db_name).expect("Failed to create database");

    // Create database tables if they don't exist
    if let Some(msg) = create_db_tables(&conn) {
        println!("{}", msg);
        if msg.starts_with("Failed") {
            return Err(msg);
        }
    }

    conn.close().expect("Failed to close database connection");

    Ok(())
}

// ----------------------------------------------------------------------------
// Create database tables if they don't exist
fn create_db_tables(conn: &Connection) -> Option<String> {
    // Return a Result, if it's an error, return the error
    // If it's Ok, do nothing

    // Check if the 'request' table exists in the database
    conn.execute(
        "CREATE TABLE IF NOT EXISTS session (
            id INTEGER PRIMARY KEY,
            start_time TEXT NOT NULL,
            yt_id TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create table 'session': {}", e))
    .ok()?;

    // Check if the 'song' table exists in the database
    conn.execute(
        "CREATE TABLE IF NOT EXISTS song (
            id INTEGER PRIMARY KEY,
            session_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            note TEXT,
            is_played INTEGER NOT NULL,
            request_id INTEGER NOT NULL,
            timestamp_id INTEGER NOT NULL,
            FOREIGN KEY (session_id) REFERENCES session (id),
            FOREIGN KEY (timestamp_id) REFERENCES timestamp (id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create table 'song': {}", e))
    .ok()?;

    // Check if the 'timestamp' table exists in the database
    conn.execute(
        "CREATE TABLE IF NOT EXISTS timestamp (
            id INTEGER PRIMARY KEY,
            song_id INTEGER NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY (song_id) REFERENCES song (id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create table 'timestamp': {}", e))
    .ok()?;


    None

}
