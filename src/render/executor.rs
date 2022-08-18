use std::future::Future;

pub struct Executor {
    #[cfg(not(target_arch = "wasm32"))]
    pool: futures::executor::ThreadPool,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            pool: futures::executor::ThreadPool::new().unwrap(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn execute<F: Future<Output = ()> + Send + 'static>(&self, f: F) {
        self.pool.spawn_ok(f);
    }
    #[cfg(target_arch = "wasm32")]
    pub fn execute<F: Future<Output = ()> + 'static>(&self, f: F) {
        wasm_bindgen_futures::spawn_local(f);
    }
}
