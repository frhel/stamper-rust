use futures_util::{StreamExt, SinkExt, stream::{SplitStream, SplitSink}};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use tokio::{sync::mpsc};

use url::Url;

use crate::TaskMessage;

struct SonglistWebsocket {
    ping_interval: i32,
    ping_interval_timer: tokio::time::Interval,
    write: SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>,
}

pub async fn connect_and_keep_alive(
    sender: mpsc::Sender<TaskMessage>,
    mut receiver: mpsc::Receiver<TaskMessage>
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
                        // Send close request to the server
                        sl_ws.write.send(Message::Text("1".to_string())).await.expect("Failed to send close request to Songlist_WS");
                        // shut down the connection loop
                        return;
                    },
                    _ => (),
                }
            }

            result = sl_ws.read.next() => {
                match result {
                    Some(Ok(message)) => {
                        // Mark pong as received
                        if message.to_string() == "3" {pong_received = true;}

                        let event_type = process_message(message.to_string()).await;

                        if event_type == "queue-update" {
                            sender.send(TaskMessage::SongQueueUpdate).await.expect("Failed to send queue update message to main thread from Songlist_WS");
                        } else if event_type == "new-playhistory" {
                            sender.send(TaskMessage::SongHistoryUpdate).await.expect("Failed to send history update message to main thread from Songlist_WS");
                        }
                    }
                    Some(Err(err)) => {
                        // Print the error and reconnect
                        println!("Error in connection to Songlist_WS: {:?}", err);
                        println!("Reconnecting in 5 seconds...");
                        sl_ws.write.send(Message::Text("1".to_string())).await.expect("Failed to send close request to Songlist_WS");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        sl_ws = setup_ws_connection().await;
                    }
                    None => {
                        println!("Songlist_WS server sent an empty packet");
                    }
                }
            }

            // ping the server on a regular interval
            _ = sl_ws.ping_interval_timer.tick() => {
                if pong_received {
                    pong_received = false;
                    // Send ping
                    sl_ws.write.send(Message::Text("2".to_string())).await.expect("Failed to send ping to Songlist_WS");
                } else {
                    println!("Songlist_WS session timed out, reconnecting in 5 seconds...");
                    // Send close request;
                    sl_ws.write.send(Message::Text("1".to_string())).await.expect("Failed to send close request to Songlist_WS");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    sl_ws = setup_ws_connection().await;
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
    // 42 is the prefix for all socket.io messages received
    if text.starts_with("42") {
        // return the whole string apart from the first 2 characters and process it
        // to be a json string
        let payload = text[2..].to_string();
        let event: Value = serde_json::from_str(&payload).expect("Failed to parse json payload from Songlist_WS");
        let event_type = event[0].as_str().unwrap_or_else(|| "");
        return event_type.to_string();
    } else {
        return String::new();
    }
}

async fn setup_ws_connection() -> SonglistWebsocket{
    let url = Url::parse("wss://api.streamersonglist.com/socket.io/?EIO=3&transport=websocket").expect("Failed to parse url for Songlist_WS connection");

    // Connect to the server using tokio_tungstenite and define a callback for received messages
    let (socket, _response) = connect_async(url).await.expect("Failed to connect to Songlist_WS");

    // Split the socket into a sender and a receiver
    let (write, read) = socket.split();

    let mut sl_ws = SonglistWebsocket {
        ping_interval: 0,
        ping_interval_timer: tokio::time::interval(tokio::time::Duration::from_millis(1000)),
        write: write,
        read: read,
    };


    sl_ws.write.send(Message::Text("42[\"join-room\",\"7325\"]".to_string())).await.expect("Failed to send join-room message to Songlist_WS");

    // Wait for the connection initialization message
    let message = sl_ws.read.next().await.expect("Failed to receive message from Songlist_WS").expect("Failed to receive message from Songlist_WS");
    let text = message.to_text().expect("Failed to convert message to text in Songlist_WS");
    if text.starts_with("0") {
        let mut handshake = text.split("{").collect::<Vec<&str>>();
        handshake = handshake[1].split("}").collect::<Vec<&str>>();
        handshake = handshake[0].split(",").collect::<Vec<&str>>();
        sl_ws.ping_interval = handshake[2].split(":").collect::<Vec<&str>>()[1].to_string().parse::<i32>().expect("Failed to parse ping_interval from Songlist_WS handshake");
        // update the ping_interval_timer with the new ping_interval
        sl_ws.ping_interval_timer = tokio::time::interval(tokio::time::Duration::from_millis(sl_ws.ping_interval as u64));
        println!("Connected to songlist ws server. Heartbeat initialized. Room Joined.");
    }

    sl_ws

}
