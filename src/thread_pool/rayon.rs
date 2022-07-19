use super::ThreadPool;


/// Wrapper of `rayon::ThreadPool`
pub struct RayonThreadPool(rayon::ThreadPool);

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> crate::Result<Self>
    where
        Self: Sized {
            Ok(RayonThreadPool(
                rayon::ThreadPoolBuilder::new().num_threads(threads as usize).build()?
            ))   
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static {
        self.0.spawn(job)
    }
}