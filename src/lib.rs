use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

struct ThreadPool {
    handlers: Vec<JoinHandle<()>>,
    sender: Sender<Box<dyn FnMut() + Send>>,
}

impl ThreadPool {
    pub fn new(num_threads: u8) -> Self {
        let mut handlers = vec![];

        let (sender, receiver) = channel::<Box<dyn FnMut() + Send>>();

        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..num_threads {
            let clone_receiver = receiver.clone();

            let handler = std::thread::spawn(move || loop {
                let mut work = match clone_receiver.lock().unwrap().recv() {
                    Ok(work) => work,
                    Err(_) => break,
                };

                println!("Starting");
                work();
                println!("Finishing");
            });

            handlers.push(handler);
        }

        ThreadPool { handlers, sender }
    }

    pub fn execute<T: FnMut() + Send + 'static>(&self, work: T) {
        let work = Box::new(work);

        self.sender.send(work).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutable_test() {
        let pool = ThreadPool::new(4);

        let mut n = Arc::new(AtomicI32::new(0));

        let n_clone = n.clone();

        let foo = move || {
            n_clone.fetch_add(15, Ordering::SeqCst);
        };

        pool.execute(foo.clone());
        pool.execute(foo);

        std::thread::sleep(std::time::Duration::from_secs(2));

        assert_eq!(n.load(Ordering::SeqCst), 30);
    }

    #[test]
    fn simple_test() {
        let pool = ThreadPool::new(4);

        pool.execute(|| {
            println!("Worker 1");
        });
        pool.execute(|| {
            println!("Worker 1");
        });

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
