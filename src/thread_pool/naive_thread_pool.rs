use crate::thread_pool::ThreadPool;

/// A 'fake' thread pool
/// this implementation is not going to reuse threads between jobs
pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> crate::Result<Self>
    where
        Self: Sized {
        Ok(NaiveThreadPool)
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static {
        std::thread::spawn(job);
    }
}