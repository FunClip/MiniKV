use std::{
    collections::VecDeque,
    panic,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::thread_pool::ThreadPool;
use crate::Result;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Task {
    Runable(Job),
    Exit,
}
/// A thread pool implemented with a shared queue
/// 
/// Note: using `catch_unwind` catch panic in thread
pub struct SharedQueueThreadPool {
    thread_handles: Vec<JoinHandle<()>>,
    task_queue: Arc<Mutex<VecDeque<Task>>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        let mut thread_handles = Vec::with_capacity(threads as usize);
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));

        for _ in 0..threads {
            let queue = Arc::clone(&task_queue);
            thread_handles.push(thread::spawn(move || task_runner(queue)));
        }

        Ok(SharedQueueThreadPool {
            thread_handles,
            task_queue,
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_queue
            .lock()
            .unwrap()
            .push_back(Task::Runable(Box::new(job)));
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.thread_handles.len() {
            self.task_queue.lock().unwrap().push_back(Task::Exit);
        }

        while !self.thread_handles.is_empty() {
            let handle = self.thread_handles.pop().unwrap();
            handle.join().unwrap();
        }
    }
}

fn task_runner(queue: Arc<Mutex<VecDeque<Task>>>) {
    loop {
        let result = panic::catch_unwind(|| {
            let option_task = queue.lock().unwrap().pop_front();
            if let Some(task) = option_task {
                match task {
                    Task::Runable(job) => {
                        job();
                        return true;
                    }
                    Task::Exit => return false,
                }
            }
            true
        });
        if result.is_err() {
            continue;
        } else if !result.unwrap() {
            break;
        }
    }
}
