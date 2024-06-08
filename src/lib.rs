// p2pass - a P2P file sharing program. each node, when invoked for the first time, will produce a unique identifier, based on their IP and port. This identifier is 2-way, and crucially is not stored in a DHT

use std::net::IpAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::vec;

// base64
use base64::{engine::general_purpose::STANDARD, Engine as _};

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

pub struct Node {
    ip: IpAddr,
    port: u16,
}

impl Node {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip,
            port,
        }
    }

    pub fn id(&self) -> String {
        let ip = self.ip.to_string();
        let port = self.port.to_string();
        let id = format!("{}:{}", ip, port);
        
        let id = STANDARD.encode(id.as_bytes());
        
        id
    }

    pub fn from(id: String) -> Self {
        let id = STANDARD.decode(id).unwrap();
        let id = String::from_utf8(id).unwrap();

        let mut parts = id.split(':');
        let ip = parts.next().unwrap().parse().unwrap();
        let port = parts.next().unwrap().parse().unwrap();
        Self {
            ip,
            port,
        }
    }

    pub fn send(&self, data: Vec<u8>) {
        // send data to the node

        // step 1: connect to the node
        // step 2: send  the data

        // connect via TCP
        let addr = format!("{}:{}", self.ip, self.port);
        let mut stream = TcpStream::connect(addr).unwrap();

        // first, send WAKEUP
        stream.write(b"WAKEUP").unwrap();
        // and flush
        stream.flush().unwrap();

        // now await the ACK response
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let mut response = String::new();

        reader.read_line(&mut response).unwrap();

        // check if the response is ACK
        if response.trim() != "ACK" {
            panic!("Expected ACK, got {}", response);
        }

        // send the data
        stream.write(&data).unwrap();
        stream.flush().unwrap();

        // now await the ACK-<hash> response
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();

        // check if the response is ACK-<hash>
        if !response.starts_with("ACK-") {
            panic!("Expected ACK-<hash>, got {}", response);
        }

        // for now, don't worry about checking the hash

        // send GOODBYE
        stream.write(b"GOODBYE").unwrap();
        stream.flush().unwrap();

        // close the connection
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    }

    pub fn listen(&self, messages: Arc<Mutex<mpsc::Sender<Vec<u8>>>>) {
        let listener = std::net::TcpListener::bind(format!("{}:{}", self.ip, self.port)).unwrap();

        for stream in listener.incoming() {
            let messages = Arc::clone(&messages);
            match stream {
                Ok(stream) => {
                    let messages = Arc::clone(&messages);
                    std::thread::spawn(move || {
                        handle_client(stream, messages);
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, messages: Arc<Mutex<mpsc::Sender<Vec<u8>>>>) {
    // first, read the WAKEUP
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut final_data = String::new();
    loop {
        let mut buffer = [0; 1024];
        let bytes_read = reader.read(&mut buffer).unwrap();

        let data = String::from_utf8_lossy(&buffer[..bytes_read]);
        final_data.push_str(&data);

        if bytes_read < 1024 {
            break;
        }
    }

    if final_data != "WAKEUP" {
        panic!("Expected WAKEUP, got {}", final_data);
    }

    // send ACK
    stream.write(b"ACK\n").unwrap();
    stream.flush().unwrap();

    // read the data
    let mut data = Vec::new();
    loop {
        let mut buffer = [0; 1024];
        let bytes_read = reader.read(&mut buffer).unwrap();

        data.extend_from_slice(&buffer[..bytes_read]);

        if bytes_read < 1024 {
            break;
        }
    }

    // utf-8 the data
    let data = String::from_utf8(data).unwrap();

    // now, sha256 that hash
    let hash = sha256::digest(data.as_bytes());

    // send ACK-<hash>
    stream.write(format!("ACK-{}\n", hash).as_bytes()).unwrap();
    stream.flush().unwrap();

    // read the GOODBYE
    let mut final_data = String::new();
    loop {
        let mut buffer = [0; 1024];
        let bytes_read = reader.read(&mut buffer).unwrap();

        let data = String::from_utf8_lossy(&buffer[..bytes_read]);
        final_data.push_str(&data);

        if bytes_read < 1024 {
            break;
        }
    }

    match final_data.trim() {
        "GOODBYE" => {
            // close the connection

            // mission accomplished! append the data to the messages
            messages
                .lock()
                .unwrap()
                .send(data.as_bytes().to_vec())
                .unwrap();
            stream.shutdown(std::net::Shutdown::Both).unwrap();
        }
        "ERR" => {
            // we failed, but shutdown the connection - the client will retry
            stream.shutdown(std::net::Shutdown::Both).unwrap();
        }
        _ => {
            panic!("Expected GOODBYE, got {}", final_data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listen() {
        let node = Node::new("127.0.0.1".parse().unwrap(), 4024);

        println!("Please use the id: {}", node.id());

        let mut messages = vec![];

        let (tx, rx) = mpsc::channel();

        let tx = Arc::new(Mutex::new(tx));

        std::thread::spawn(move || {
            node.listen(tx);
        });

        while let Ok(message) = rx.recv() {
            messages.push(message.clone());

            println!("Received message: {:?}", message);
        }
    }

    #[test]
    fn test_send() {
        let mut id: String = String::new();

        let _ = std::io::stdin().read_line(&mut id).unwrap();

        let node = Node::from(id.trim().to_string());

        println!("hello!");

        node.send(b"Hello, world!\n".to_vec());
    }
}
