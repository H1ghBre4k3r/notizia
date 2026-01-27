use notizia::prelude::*;

#[derive(Task)]
#[task(message = PingMsg)]
struct PingProc;

#[derive(Debug, Clone)]
struct PingMsg;

impl Runnable<PingMsg> for PingProc {
    async fn start(&self) {
        println!("Starting PingProc");
        let pong_proc = spawn!(PongProc);

        for i in 0..10 {
            send!(pong_proc, PongMsg(self.this())).expect("Sending should work");

            let msg = recv!(self).unwrap();
            println!("PingProc received: {msg:?} #{i}");
        }
        pong_proc.kill();
    }
}

#[derive(Task)]
#[task(message = PongMsg)]
struct PongProc;

#[derive(Debug, Clone)]
struct PongMsg(TaskRef<PingMsg>);

impl Runnable<PongMsg> for PongProc {
    async fn start(&self) {
        println!("Starting PongProc");
        loop {
            let msg = recv!(self).unwrap();
            println!("PongProc received {msg:?}");
            let PongMsg(other) = msg;
            send!(other, PingMsg).expect("Sending should work");
        }
    }
}

#[tokio::main]
async fn main() {
    let task = PingProc;

    let handle = spawn!(task);

    match handle.join().await {
        Ok(reason) => println!("Terminated: {reason}"),
        Err(e) => eprintln!("{e}"),
    }
}
