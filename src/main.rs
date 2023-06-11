use tokio::{sync::mpsc, runtime::Runtime};

mod models;
use crate::models::{update_active_song, CurrentSong};//, DbModel};

mod songlist_ws;
use crate::songlist_ws::connect_and_keep_alive;

mod session_controller;


#[derive(Debug, Clone)]
pub enum TaskMessage {
    Shutdown,
    SongQueueUpdate,
    SongHistoryUpdate,
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async_main());
}

pub async fn async_main() {
    let mut current_song = CurrentSong { ..Default::default() };
    current_song = update_active_song(current_song).await;

    // Initialize the session
    let _session = session_controller::init_session();

    // Create the channels to communicate between the threads
    let (main_sender, mut main_receiver) = mpsc::channel(10);
    let (connection_sender, connection_receiver) = mpsc::channel(10);

    // Spawn the threads
    let sl_ws_task = tokio::spawn(connect_and_keep_alive(
        main_sender.clone(),
        connection_receiver
    ));

    // Handle signals and messages in the main task
    let mut shutdown = false;

    while !shutdown {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl-C received, sending shutdown signal...");
                connection_sender.send(TaskMessage::Shutdown).await.unwrap();
                shutdown = true;
            }

            msg = main_receiver.recv() => {
                match msg {
                    Some(TaskMessage::SongQueueUpdate) => {
                        println!("Received song queue update");
                        current_song = update_active_song(current_song).await;
                        println!("Current song: {:?}", current_song)
                    },
                    Some(TaskMessage::SongHistoryUpdate) => {
                        println!("Received song history update");
                    },
                    _ => (),
                }
            }
        }

        // sleep for 1 second
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Wait for the threads to finish
    sl_ws_task.await.unwrap();

    println!("Exiting...");

}



