use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread::JoinHandle,
};

#[derive(Clone)]
struct Mailbox<T>(Sender<T>);

pub struct Task<M, R> {
    mailbox: Mailbox<M>,
    handle: JoinHandle<R>,
}

impl<T, R> Task<T, R>
where
    T: Clone,
{
    pub fn send(&self, payload: T) {
        self.mailbox.0.send(payload).unwrap()
    }

    pub fn join(self) -> R {
        self.handle.join().unwrap()
    }
}

#[macro_export]
macro_rules! proc {
    ($($content:tt)*) => {
        notizia::spawn_task(move |_receiver| {
            #[allow(unused_macros)]
            macro_rules! recv {
                () => { _receiver.recv().unwrap() }
            }
            $($content)*
        })
    };
}

pub fn spawn_task<M, R, Func>(func: Func) -> Task<M, R>
where
    M: Send + 'static,
    R: Send + 'static,
    Func: FnOnce(Receiver<M>) -> R + Send + 'static,
{
    let (sender, receiver) = channel::<M>();
    let mb = Mailbox(sender);
    let handle = std::thread::spawn(move || func(receiver));

    Task {
        mailbox: mb,
        handle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_task_communication() {
        let task = spawn_task(|receiver| {
            let mut total = 0;
            for _ in 0..3 {
                total += receiver.recv().unwrap();
            }
            total
        });

        task.send(10);
        task.send(20);
        task.send(30);

        let result = task.join();
        assert_eq!(result, 60);
    }

    #[test]
    fn test_sum_of_five_numbers() {
        let task = spawn_task(|receiver| {
            let mut total = 0;
            for _ in 0..5 {
                total += receiver.recv().unwrap();
            }
            total
        });

        for i in 1..=5 {
            task.send(i);
        }

        let result = task.join();
        assert_eq!(result, 15);
    }

    #[test]
    fn test_spawn_task_with_multiple_messages() {
        let task = spawn_task(|receiver| {
            let mut sum = 0;
            for _ in 0..3 {
                sum += receiver.recv().unwrap();
            }
            sum
        });

        task.send(5);
        task.send(10);
        task.send(15);

        let result = task.join();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_string_messages() {
        let task = spawn_task(|receiver| {
            let mut count = 0;
            for _ in 0..3 {
                receiver.recv().unwrap();
                count += 1;
            }
            count
        });

        task.send("hello".to_string());
        task.send("world".to_string());
        task.send("test".to_string());

        let result = task.join();
        assert_eq!(result, 3);
    }

    #[test]
    fn test_task_returns_vec() {
        let task = spawn_task(|receiver| {
            let mut values = Vec::new();
            for _ in 0..5 {
                let val = receiver.recv().unwrap();
                values.push(val);
            }
            values
        });

        for i in 1..=5 {
            task.send(i);
        }

        let result = task.join();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_empty_task() {
        let task = spawn_task::<(), u32, _>(|_receiver| 42);

        let result = task.join();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_different_number_types() {
        let task = spawn_task(|receiver| {
            let mut sum: i64 = 0;
            for _ in 0..3 {
                let val: i32 = receiver.recv().unwrap();
                sum += val as i64;
            }
            sum
        });

        task.send(100);
        task.send(200);
        task.send(300);

        let result = task.join();
        assert_eq!(result, 600);
    }
}
