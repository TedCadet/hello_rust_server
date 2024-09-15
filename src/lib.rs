use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver_ref: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread: JoinHandle<()> = thread::spawn(move || loop {
            let message: Result<Box<dyn FnOnce() + Send>, mpsc::RecvError> =
                receiver_ref.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    sender: Option<Sender<Job>>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    ///Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver_ref: Arc<Mutex<Receiver<Job>>> = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker> = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver_ref)));
        }

        ThreadPool {
            sender: Some(sender),
            workers,
        }
    }

    // pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {}

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Box::new(f);

        println!("number of workers: {}", self.workers.len());

        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .unwrap_or_else(|err| println!("erreur lors de send la job au channel: '{err}'"));
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
