//! Supervisor pattern example.
//!
//! This example demonstrates:
//! - A supervisor managing multiple worker tasks
//! - Dynamic task spawning
//! - Coordinated shutdown

use notizia::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone)]
enum WorkerMsg {
    Work(u32),
    Stop,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum SupervisorMsg {
    DistributeWork(u32),
    WorkerFinished { id: usize, count: u32 },
    Stop,
}

#[derive(Task)]
#[task(message = WorkerMsg)]
struct Worker {
    id: usize,
    processed: Arc<AtomicU32>,
}

impl Runnable<WorkerMsg> for Worker {
    async fn start(&self) {
        println!("Worker {} started", self.id);
        let mut count = 0u32;

        loop {
            match recv!(self) {
                Ok(WorkerMsg::Work(value)) => {
                    println!("Worker {} processing: {}", self.id, value);
                    // Simulate work
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    count += 1;
                    self.processed.fetch_add(value, Ordering::SeqCst);
                }
                Ok(WorkerMsg::Stop) => {
                    println!("Worker {} stopping (processed {} items)", self.id, count);
                    break;
                }
                Err(_) => break,
            }
        }
    }
}

#[derive(Task)]
#[task(message = SupervisorMsg)]
struct Supervisor {
    num_workers: usize,
}

impl Runnable<SupervisorMsg> for Supervisor {
    async fn start(&self) {
        println!("Supervisor starting with {} workers\n", self.num_workers);

        // Spawn workers
        let mut workers = Vec::new();
        let processed = Arc::new(AtomicU32::new(0));

        for id in 0..self.num_workers {
            let worker = Worker {
                id,
                processed: processed.clone(),
            };
            let handle = spawn!(worker);
            workers.push(handle);
        }

        let mut next_worker = 0;

        loop {
            match recv!(self) {
                Ok(SupervisorMsg::DistributeWork(value)) => {
                    // Round-robin distribution
                    let worker = &workers[next_worker];
                    worker.send(WorkerMsg::Work(value)).unwrap();
                    next_worker = (next_worker + 1) % self.num_workers;
                }
                Ok(SupervisorMsg::WorkerFinished { id, count }) => {
                    println!("Supervisor: Worker {} finished with {} items", id, count);
                }
                Ok(SupervisorMsg::Stop) => {
                    println!("\nSupervisor initiating shutdown...");

                    // Stop all workers
                    for worker in &workers {
                        worker.send(WorkerMsg::Stop).unwrap();
                    }

                    // Wait for all workers to finish
                    for (id, worker) in workers.into_iter().enumerate() {
                        worker.join().await;
                        println!("Supervisor: Worker {} joined", id);
                    }

                    println!("Total processed: {}", processed.load(Ordering::SeqCst));
                    break;
                }
                Err(_) => break,
            }
        }

        println!("Supervisor finished");
    }
}

#[tokio::main]
async fn main() {
    println!("=== Supervisor Pattern Example ===\n");

    // Create supervisor with 3 workers
    let supervisor = Supervisor { num_workers: 3 };
    let handle = spawn!(supervisor);

    // Distribute work
    println!("Distributing work...\n");
    for i in 1..=12 {
        handle.send(SupervisorMsg::DistributeWork(i)).unwrap();
    }

    // Let workers process
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Shutdown
    handle.send(SupervisorMsg::Stop).unwrap();
    handle.join().await;

    println!("\nAll done!");
}
