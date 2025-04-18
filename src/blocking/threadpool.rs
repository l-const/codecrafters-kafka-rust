use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

type WorkerStatus = AtomicBool;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>, status: Arc<WorkerStatus>) -> Worker {
        let thread = thread::spawn(move || loop {
            status.store(false, Ordering::Relaxed);
            match receiver.lock().unwrap().recv() {
                Ok(job) => {
                    status.store(true, Ordering::Relaxed);
                    println!("Worker {} got a job; executing.", id);
                    job();
                }

                Err(_) => {
                    println!("Worker {} disconnected; shutting down.", id);
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

// type alias for the job closure
type Job = Box<dyn FnOnce() + Send + 'static>;

// ThreadPool struct
pub struct ThreadPool {
    workers: Vec<(Worker, Arc<WorkerStatus>)>,
    senders: Option<Vec<(WorkerId, Sender<Job>)>>,
}

type WorkerId = usize;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size);
        let mut senders = Vec::with_capacity(size);
        for id in 0..size {
            let (sender, receiver) = channel();
            let receiver = Arc::new(Mutex::new(receiver));
            let worker_status = Arc::new(AtomicBool::new(false));
            workers.push((
                Worker::new(id, Arc::clone(&receiver), worker_status.clone()),
                worker_status,
            ));
            senders.push((id, sender));
        }

        ThreadPool {
            workers,
            senders: Some(senders),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender to signal termination to workers

        drop(self.senders.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.0.id);
            if let Some(thread) = worker.0.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl ThreadPool {
    pub fn execute<F>(&self, f: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let workers_available: Option<WorkerId> = self
            .workers
            .iter()
            .filter(|(_, status)| status.load(Ordering::Relaxed) == false)
            .map(|(worker, _)| worker.id)
            .find(|_| true);
        if let Some(id) = workers_available {
            let sender_avail = self
                .senders
                .as_ref()
                .unwrap()
                .iter()
                .find(|(id_send, _)| *id_send == id);
            sender_avail.unwrap().1.send(job).unwrap();
            Ok(())
        } else {
            Err("no available workers".to_string().into())
        }
    }
}
