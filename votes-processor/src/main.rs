use queue_client::Queue;
use std::io::Read;

fn main() {
    let mut queue = Queue::new("127.0.0.1:8082").unwrap();

    queue.subscribe();

    loop {
        let mut message = vec![0; 1024];
        let bytes_read = queue.connection.read(&mut message).unwrap();

        if bytes_read == 0 {
            println!("Connection closed");
            break;
        }

        let message = std::str::from_utf8(&message[..bytes_read]).unwrap();

        println!("Received: {:?}", message);
    }
}
