use std::thread;
use std::sync::{mpsc, Arc};
use std::sync::mpsc::{Sender, Receiver};
use rustls::lock::Mutex;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        Worker {
            id,
            thread: Some(thread::spawn(move || {
                loop {
                    let guard = reciever.lock();
                    match guard {
                        Some(r) => match r.recv() {
                            Ok(w) => {
                                drop(r);
                                w();
                            },
                            Err(mpsc::RecvError) => {
                                drop(r);
                                return;
                            },
                        },
                        None => break,
                    }
                }
            })),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        let (sender, reciever): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let rec_arc = Arc::new(Mutex::new(reciever));

        for idx in 0..size {
            let worker = Worker::new(idx, rec_arc.clone());
            workers.push(worker);
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            let w = worker.thread.take();
            match w {
                Some(work) => {
                    let _ = work.join();
                },
                None => (),
            }
        }
    }
}
