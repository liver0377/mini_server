use std::thread;
use std::sync::{Arc, Mutex, mpsc};



// once a worker is created by Worker::new()
// it holds a running thread trying to get the mutex, which protects the receive end of channel
// when worker get the mutex, it will receive the job from the receive end of chnnel, then do it and loop this procedure
#[allow(unused)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // lock() may return None when it is posioned state: some other thread panicked when holding the lock 
            // recv() will block until sender sends job here
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");
            job();
        });

        Worker {
            id,
            thread,
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[allow(unused)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
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
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool  { workers, sender }
    }

    pub fn execute<F>(&self, f: F) 
        where F: FnOnce() + Send + 'static {
            let job = Box::new(f);

            self.sender.send(job).unwrap();
    }
}


#[cfg(test)]
mod tests {

}