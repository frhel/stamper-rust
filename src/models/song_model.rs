use serde::{Deserialize, Serialize};

// https://api.streamersonglist.com/docs/ for endpoints.
// Numerical value of endpoint - Absolutely no clue where I found this to begin with
const STREAMER_ID: &str = "7325";
const SONGLIST_APIURI: &str = "https://api.streamersonglist.com/v1/streamers/";
const SONGLIST_APIQUEUE_ENDPOINT: &str = "/queue";
const _SONGLIST_APIHISTORY_ENDPOINT: &str = "/playHistory?period=stream";

#[derive(Debug, Serialize, Deserialize)]
struct Queue {
    list: Vec<QueueItem>,
    status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueueItem {
    id: i32,
    note: String,
    #[serde(rename = "botRequestBy")]
    bot_request_by: Option<String>,
    #[serde(rename = "nonlistSong")]
    nonlist_song: Option<String>,
    #[serde(rename = "donationAmount")]
    donation_amount: i32,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "songId")]
    song_id: Option<i32>,
    #[serde(rename = "streamerId")]
    streamer_id: i32,
    song: Option<Song>,
    requests: Vec<Requests>,
    position: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Song {
    id: i32,
    title: String,
    artist: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    comment: Option<String>,
    capo: String,
    #[serde(rename = "attributeIds")]
    attribute_ids: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Requests{
    id: i32,
    name: String,
    note: Option<String>,
    amount: i32,
    source: String,
    #[serde(rename = "inChat")]
    in_chat: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Status {
    #[serde(rename = "songsPlayedToday")]
    songs_played_today: i32,
}

#[derive(Debug, Clone)]
pub struct CurrentSong {
    pub title: String,
    pub artist: String,
    pub note: String,
    pub is_played: bool,
    pub request_id: i32,
    pub timestamp_index: i32,
    pub timestamps: Vec<i32>,
}

impl Default for CurrentSong {
    fn default() -> Self {
        CurrentSong {
            title: String::new(),
            artist: String::new(),
            note: String::new(),
            is_played: false,
            request_id: 0,
            timestamp_index: 0,
            timestamps: Vec::new(),
        }
    }
}

pub async fn update_active_song(current_song: CurrentSong) -> CurrentSong {
    let old_song = current_song.clone();

    let list = get_song_queue().await.unwrap_or_else(|err| {
        println!("Error: {}", err);
        Vec::new()
    });

    // if list is not empty, process the first item
    let new_song =  if !list.is_empty() {
        process_song(&list[0], old_song)
    } else {
        old_song
    };
    new_song
}

fn process_song(item: &QueueItem, mut old_song: CurrentSong) -> CurrentSong {
    // Check if song is not null
    if let Some(song) = &item.song {
        old_song.title = song.title.clone();
        old_song.artist = song.artist.clone();
    } else {
        if item.nonlist_song.is_some() {
            // split string by " - " and collect into a vector
            let split: Vec<&str> = item.nonlist_song.as_ref().unwrap().split(" - ").collect();
            old_song.title = split[0].to_string();
            old_song.artist = split[1].to_string();
        }
    }

    old_song.request_id = item.id.clone();
    old_song.note = item.note.clone();

    old_song
}

async fn get_song_queue() -> Result<Vec<QueueItem>, Box<dyn std::error::Error>> {
    let songlist_apiqueue_uri: String = format!("{}{}{}",  SONGLIST_APIURI, STREAMER_ID, SONGLIST_APIQUEUE_ENDPOINT);
    let resp: Queue= reqwest::get(&songlist_apiqueue_uri).await?.json().await?;

    Ok(resp.list)
}