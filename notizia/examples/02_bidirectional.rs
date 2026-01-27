//! Bidirectional communication example.
//!
//! This example demonstrates:
//! - Two tasks communicating with each other
//! - Using TaskRef for lightweight references
//! - Request-response patterns

use notizia::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ClientMsg {
    Request(u32),
    Response(u32),
    Stop,
}

#[derive(Debug, Clone)]
enum ServerMsg {
    Process {
        value: u32,
        reply_to: TaskRef<ClientMsg>,
    },
    Stop,
}

#[derive(Task)]
#[task(message = ClientMsg)]
struct Client {
    id: usize,
    responses: Arc<AtomicU32>,
}

impl Runnable<ClientMsg> for Client {
    async fn start(&self) {
        println!("Client {} started", self.id);

        loop {
            match recv!(self) {
                Ok(ClientMsg::Request(value)) => {
                    println!("Client {} received request with value: {}", self.id, value);
                }
                Ok(ClientMsg::Response(result)) => {
                    println!("Client {} received response: {}", self.id, result);
                    self.responses.fetch_add(1, Ordering::SeqCst);
                }
                Ok(ClientMsg::Stop) => {
                    println!("Client {} stopping", self.id);
                    break;
                }
                Err(_) => break,
            }
        }
    }
}

#[derive(Task)]
#[task(message = ServerMsg)]
struct Server {
    processed: Arc<AtomicU32>,
}

impl Runnable<ServerMsg> for Server {
    async fn start(&self) {
        println!("Server started");

        loop {
            match recv!(self) {
                Ok(ServerMsg::Process { value, reply_to }) => {
                    println!("Server processing value: {}", value);

                    // Simulate processing
                    let result = value * 2;

                    // Send response back to client
                    if let Err(e) = reply_to.send(ClientMsg::Response(result)) {
                        eprintln!("Failed to send response: {}", e);
                    } else {
                        self.processed.fetch_add(1, Ordering::SeqCst);
                    }
                }
                Ok(ServerMsg::Stop) => {
                    println!("Server stopping");
                    break;
                }
                Err(_) => break,
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Bidirectional Communication Example ===\n");

    let processed = Arc::new(AtomicU32::new(0));
    let responses = Arc::new(AtomicU32::new(0));

    // Start server
    let server = Server {
        processed: processed.clone(),
    };
    let server_handle = spawn!(server);

    // Start client
    let client = Client {
        id: 1,
        responses: responses.clone(),
    };
    let client_handle = spawn!(client);
    let client_ref = client_handle.this();

    // Send requests from client to server
    println!("\nSending requests...\n");
    for value in 1..=5 {
        server_handle
            .send(ServerMsg::Process {
                value,
                reply_to: client_ref.clone(),
            })
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Wait a bit for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("\nShutting down...\n");

    // Shutdown
    server_handle.send(ServerMsg::Stop).unwrap();
    client_handle.send(ClientMsg::Stop).unwrap();

    let _ = server_handle.join().await;
    let _ = client_handle.join().await;

    println!("\nStatistics:");
    println!(
        "  Server processed: {} requests",
        processed.load(Ordering::SeqCst)
    );
    println!(
        "  Client received: {} responses",
        responses.load(Ordering::SeqCst)
    );
}
