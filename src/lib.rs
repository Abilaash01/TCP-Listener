use std::{sync::{mpsc::{self}, Arc, Mutex}, thread};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message    {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Creates a new Threadpool
    /// 
    /// The size is the number of threads in the pool
    /// 
    /// # Panics
    /// 
    /// The 'new' function will panic when pool size is zero
    /// 
    /// # Arguments
    /// 
    /// * `size` - The size of the pool
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // Create thread
            workers.push(Worker::new(
                id, 
                Arc::clone(&receiver)
            ));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        // Send job to worker
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Drop implementation for threadpool
    /// 
    /// Terminates all threads in the pool
    /// 
    /// # Panics
    /// 
    /// The 'drop' function will panic when the threadpool is dropped and there are still active workers
    fn drop(&mut self) {
        print!("Sending terminate message to all workers.");

        // Send terminate message to all workers
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        // Join all workers
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}


struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    /// Creates a new worker
    /// 
    /// # Panics
    /// 
    /// The 'new' function will panic when the id is zero
    /// 
    /// # Arguments
    /// 
    /// * `id` - The id of the worker
    /// * `receiver` - The receiver of the job
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver
            .lock()
            .unwrap()
            .recv()
            .unwrap();

            println!("Worker {} got a job; executing.", id);

            // Execute job or terminate depending on message
            match message {
                Message::NewJob(job) => {
                    print!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            };
        });

        // Return worker
        Worker {id, thread: Some(thread)}
    }
}