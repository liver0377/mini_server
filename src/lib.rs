use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

// once a worker is created by Worker::new()
// it holds a running thread trying to get the mutex, which protects the receive end of channel
// when worker get the mutex, it will receive the job from the receive end of chnnel, then do it and loop this procedure
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // lock() may return None when it is posioned state: some other thread panicked when holding the lock
            // recv() will block until sender sends job here
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("worker {id} got a job; executing");

                    job();
                },
                _ => {
                    println!("Worker {id} disconnected; shutting down");
                    break ;
                }
            }
        });

        let thread = Some(thread);

        Worker { id, thread }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new thread pool
    ///
    /// the `sizes` is the num of the threads in the pool
    ///
    /// # Panics
    /// the `new` function will panic if the size is less or equal than zero
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();
        let sender = Some(sender);
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // drop the sender first, so workers will not receive more request
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
