use tokio::sync::{mpsc};

use serde_json::Value;
use tokio::{runtime::Runtime};

use url::Url;

use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};


mod models;
use crate::models::{update_active_song, CurrentSong, init_session};//, DbModel};

struct SonglistWebsocket {
    ping_interval: i32,
    ping_interval_timer: tokio::time::Interval,
    write: SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>,
}

#[derive(Debug, Clone)]
enum TaskMessage {
    Shutdown,
    SongQueueUpdate,
    SongHistoryUpdate,
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let mut current_song = CurrentSong { ..Default::default() };
    current_song = update_active_song(current_song).await;

    // Initialize the session
    let session = init_session();

    // Create the channels to communicate between the threads
    let (main_sender, mut main_receiver) = mpsc::channel(10);
    let (connection_sender, connection_receiver) = mpsc::channel(10);

    // Spawn the threads
    let connection_task = tokio::spawn(connection_loop(
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
    connection_task.await.unwrap();

    println!("Exiting...");

}


async fn connection_loop(
    sender: tokio::sync::mpsc::Sender<TaskMessage>,
    mut receiver: tokio::sync::mpsc::Receiver<TaskMessage>
) {
    let mut sl_ws = setup_ws_connection().await;
    let mut pong_received = true;
    // Websocket connection loop
    loop {
        tokio::select! {
            // handle messages from the main thread
            msg = receiver.recv() => {
                match msg {
                    Some(TaskMessage::Shutdown) => {
                        println!("Received shutdown signal");
                        sl_ws.write.send(Message::Text("1".to_string())).await.unwrap();
                        // shut down the connection loop
                        return;
                    },
                    _ => (),
                }
            }

            result = sl_ws.read.next() => {
                match result {
                    Some(Ok(message)) => {
                        if message.to_string() == "3" {pong_received = true;}

                        let (event_type) = process_message(message.to_string()).await;

                        if event_type == "queue-update" {
                            sender.send(TaskMessage::SongQueueUpdate).await.unwrap();
                        } else if event_type == "new-playhistory" {
                            sender.send(TaskMessage::SongHistoryUpdate).await.unwrap();
                        }
                    }
                    Some(Err(err)) => {
                        // Print the error and reconnect
                        println!("Error in connection: {:?}", err);
                        println!("Reconnecting...");
                        sl_ws.write.send(Message::Text("1".to_string())).await.unwrap();
                        sl_ws = setup_ws_connection().await;
                    }
                    None => {
                        println!("Server sent an empty packet");
                        sl_ws.write.send(Message::Text("3".to_string())).await.unwrap();
                    }
                }
            }

            // ping the server on a regular interval
            _ = sl_ws.ping_interval_timer.tick() => {
                if pong_received {
                    pong_received = false;
                    sl_ws.write.send(Message::Text("2".to_string())).await.unwrap();
                } else {
                    println!("Connection has timed out, reconnecting...");
                    sl_ws.write.send(Message::Text("1".to_string())).await.unwrap();
                    sl_ws = setup_ws_connection().await;
                    sl_ws.write.send(Message::Text("2".to_string())).await.unwrap();
                }
            }
        }

        // timer to only run the loop every 1 second
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}


async fn process_message(message: String) -> String {
    // take the string from the message and convert the payload to a string
    let text = message;
    // split the string by "42" and collect into a vector
    if text.starts_with("42") {
        // return the whole string apart from the first 2 characters and process it
        // to be a json string
        let packet = text.split("42").collect::<Vec<&str>>();
        let event: Value = serde_json::from_str(packet[1]).unwrap();
        let event_type = event[0].as_str().unwrap();
        return event_type.to_string();
    } else {
        return String::new();
    }
}

async fn setup_ws_connection() -> SonglistWebsocket{
    let url = Url::parse("wss://api.streamersonglist.com/socket.io/?EIO=3&transport=websocket").unwrap();

    // Connect to the server using tokio_tungstenite and define a callback for received messages
    let (socket, _response) = connect_async(url).await.expect("Failed to connect");

    // Split the socket into a sender and a receiver
    let (write, read) = socket.split();

    let mut sl_ws = SonglistWebsocket {
        ping_interval: 0,
        ping_interval_timer: tokio::time::interval(tokio::time::Duration::from_millis(1000)),
        write: write,
        read: read,
    };


    sl_ws.write.send(Message::Text("42[\"join-room\",\"7325\"]".to_string())).await.unwrap();

    // Wait for the connection initialization message
    let message = sl_ws.read.next().await.unwrap().unwrap();
    let text = message.to_text().unwrap();
    if text.starts_with("0") {
        let mut handshake = text.split("{").collect::<Vec<&str>>();
        handshake = handshake[1].split("}").collect::<Vec<&str>>();
        handshake = handshake[0].split(",").collect::<Vec<&str>>();
        sl_ws.ping_interval = handshake[2].split(":").collect::<Vec<&str>>()[1].to_string().parse::<i32>().unwrap();
        // update the ping_interval_timer with the new ping_interval
        sl_ws.ping_interval_timer = tokio::time::interval(tokio::time::Duration::from_millis(sl_ws.ping_interval as u64));
        println!("Connected to songlist ws server. Heartbeat initialized. Room Joined.");
    }

    sl_ws

}

