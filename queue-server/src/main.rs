use log::{debug, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
enum MessageType {
    Publish,
    Subscribe,
}

impl MessageType {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::Publish,
            1 => Self::Subscribe,
            _ => panic!("Invalid message type"),
        }
    }
}

type SharedState = Arc<Mutex<State>>;
type Consumers = Arc<Mutex<HashMap<String, TcpStream>>>;
type Queue = Arc<Mutex<VecDeque<Vec<u8>>>>;

#[derive(Clone)]
struct State {
    queue: Queue,
    consumers: Consumers,
}

fn new_state() -> SharedState {
    Arc::new(Mutex::new(State {
        queue: new_queue(),
        consumers: new_consumers(),
    }))
}

fn new_queue() -> Queue {
    Arc::new(Mutex::new(VecDeque::new()))
}

fn new_consumers() -> Consumers {
    Arc::new(Mutex::new(HashMap::new()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    info!("Starting queue");

    let addr = "127.0.0.1:8012";

    let listener = Arc::new(TcpListener::bind(addr).await?);

    info!("Queue Listening on: {}", addr);

    let state = new_state();

    let state_distribution = Arc::clone(&state);

    tokio::spawn(async move {
        loop {
            let state = Arc::clone(&state_distribution);

            debug!("Distributing messages");
            distribute_messages(state).await;

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    loop {
        let (socket, addr) = listener.accept().await?;

        debug!("Accepted connection from: {}", addr);

        let state = Arc::clone(&state);

        tokio::spawn(async move {
            handle_connection(state, (socket, addr)).await;
        });
    }
}

async fn handle_connection(state: SharedState, (mut socket, addr): (TcpStream, SocketAddr)) {
    loop {
        let mut buf = [0; 1024];
        let buf_len = socket.read(&mut buf).await.unwrap();

        debug!("Received {} bytes, from {}", buf_len, addr);

        if buf_len == 0 {
            info!("Connection closed with: {}", addr);
            break;
        }

        let message_type = MessageType::from_byte(buf[0]);

        info!("Received connection of: {:?}", message_type);

        match message_type {
            MessageType::Publish => {
                let state = state.lock().await;
                let mut queue = state.queue.lock().await;
                debug!("Received message: {:?}", &buf[1..buf_len]);
                queue.push_back(buf[1..buf_len].to_vec());
            }
            MessageType::Subscribe => {
                let state = state.lock().await;
                let mut consumers = state.consumers.lock().await;
                debug!("Subscribing consumer: {}", addr);
                consumers.insert(addr.to_string(), socket);
                break;
            }
        }
    }
}

async fn distribute_messages(state: SharedState) {
    let state = state.lock().await;
    let mut queue = state.queue.lock().await;

    if queue.is_empty() {
        debug!("Queue is empty, skipping distribution");
        return;
    }

    let mut consumers = state.consumers.lock().await;

    if consumers.is_empty() {
        debug!("No consumers available, skipping distribution");
        return;
    }

    let message = queue.pop_front().unwrap();

    debug!(
        "Distributing message: {:?}",
        std::str::from_utf8(&message).unwrap()
    );

    for (addr, consumer) in consumers.iter_mut() {
        debug!("Sending message to consumer: {}", addr);
        consumer.write_all(&message).await.unwrap();
    }
}
