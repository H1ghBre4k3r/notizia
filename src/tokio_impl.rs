use std::future::Future;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task::JoinHandle,
};

#[derive(Clone)]
struct AsyncMailbox<T>(UnboundedSender<T>);

pub struct AsyncTask<M, R> {
    mailbox: AsyncMailbox<M>,
    handle: JoinHandle<R>,
}

impl<T, R> AsyncTask<T, R>
where
    T: Clone,
{
    pub async fn send(&self, payload: T) {
        self.mailbox.0.send(payload).unwrap()
    }

    pub async fn join(self) -> R {
        self.handle.await.unwrap()
    }
}

#[macro_export]
macro_rules! async_proc {
    ($($content:tt)*) => {
        notizia::spawn_async_task(move |mut _receiver| async move {
            #[allow(unused_macros)]
            macro_rules! recv {
                () => { _receiver.recv().await.unwrap() }
            }
            $($content)*
        })
    };
}

pub fn spawn_async_task<M, R, Output, Func>(func: Func) -> AsyncTask<M, Output>
where
    M: Send + 'static,
    R: Send + 'static + Future<Output = Output>,
    Output: Send + 'static,
    Func: FnOnce(UnboundedReceiver<M>) -> R + Send + 'static,
{
    let (sender, receiver) = unbounded_channel::<M>();
    let mb = AsyncMailbox(sender);
    let handle = tokio::spawn(func(receiver));

    AsyncTask {
        mailbox: mb,
        handle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_async_task_communication() {
        let task = spawn_async_task(|mut receiver| async move {
            let mut total = 0;
            for _ in 0..3 {
                total += receiver.recv().await.unwrap();
            }
            total
        });

        task.send(10).await;
        task.send(20).await;
        task.send(30).await;

        let result = task.join().await;
        assert_eq!(result, 60);
    }

    #[tokio::test]
    async fn test_async_sum_of_five_numbers() {
        let task = spawn_async_task(|mut receiver| async move {
            let mut total = 0;
            for _ in 0..5 {
                total += receiver.recv().await.unwrap();
            }
            total
        });

        for i in 1..=5 {
            task.send(i).await;
        }

        let result = task.join().await;
        assert_eq!(result, 15);
    }

    #[tokio::test]
    async fn test_spawn_async_task_with_multiple_messages() {
        let task = spawn_async_task(|mut receiver| async move {
            let mut sum = 0;
            for _ in 0..3 {
                sum += receiver.recv().await.unwrap();
            }
            sum
        });

        task.send(5).await;
        task.send(10).await;
        task.send(15).await;

        let result = task.join().await;
        assert_eq!(result, 30);
    }

    #[tokio::test]
    async fn test_async_string_messages() {
        let task = spawn_async_task(|mut receiver| async move {
            let mut count = 0;
            for _ in 0..3 {
                receiver.recv().await.unwrap();
                count += 1;
            }
            count
        });

        task.send("hello".to_string()).await;
        task.send("world".to_string()).await;
        task.send("test".to_string()).await;

        let result = task.join().await;
        assert_eq!(result, 3);
    }

    #[tokio::test]
    async fn test_async_task_returns_vec() {
        let task = spawn_async_task(|mut receiver| async move {
            let mut values = Vec::new();
            for _ in 0..5 {
                let val = receiver.recv().await.unwrap();
                values.push(val);
            }
            values
        });

        for i in 1..=5 {
            task.send(i).await;
        }

        let result = task.join().await;
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_async_empty_task() {
        let task = spawn_async_task(
            |_receiver: tokio::sync::mpsc::UnboundedReceiver<()>| async move { 42 },
        );

        let result = task.join().await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_async_different_number_types() {
        let task = spawn_async_task(|mut receiver| async move {
            let mut sum: i64 = 0;
            for _ in 0..3 {
                let val: i32 = receiver.recv().await.unwrap();
                sum += val as i64;
            }
            sum
        });

        task.send(100).await;
        task.send(200).await;
        task.send(300).await;

        let result = task.join().await;
        assert_eq!(result, 600);
    }

    #[tokio::test]
    async fn test_async_task_with_tokio_sleep() {
        let task = spawn_async_task(|mut receiver| async move {
            let mut total = 0;
            for _ in 0..3 {
                total += receiver.recv().await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            total
        });

        task.send(10).await;
        task.send(20).await;
        task.send(30).await;

        let result = task.join().await;
        assert_eq!(result, 60);
    }

    #[tokio::test]
    async fn test_multiple_async_tasks() {
        let task1 = spawn_async_task(|mut receiver| async move {
            let mut total = 0;
            for _ in 0..3 {
                total += receiver.recv().await.unwrap();
            }
            total
        });

        let task2 = spawn_async_task(|mut receiver| async move {
            let mut total = 0;
            for _ in 0..3 {
                let val = receiver.recv().await.unwrap();
                total = total * 2 + val;
            }
            total
        });

        task1.send(10).await;
        task2.send(10).await;
        task1.send(20).await;
        task2.send(20).await;
        task1.send(30).await;
        task2.send(30).await;

        let result1 = task1.join().await;
        let result2 = task2.join().await;

        assert_eq!(result1, 60);
        assert_eq!(result2, 110); // ((0*2)+10)=10, ((10*2)+20)=40, ((40*2)+30)=110
    }
}
