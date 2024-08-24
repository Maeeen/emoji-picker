use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::JoinHandle};

struct Poller {
    handle: JoinHandle<()>,
    semaphore: Arc<AtomicBool>
}

impl Poller {
    pub fn new(mut f: impl FnMut() + Send + 'static) -> Self {
        let arc = Arc::new(AtomicBool::new(false));
        let arc_cloned = arc.clone();
        Poller { 
            handle: std::thread::spawn(move || loop {
                if arc_cloned.load(Ordering::Acquire) { break }
                f();
                std::thread::sleep(std::time::Duration::from_millis(100));
            }),
            semaphore: arc
        }
    }

    pub fn join(self) {
        self.semaphore.store(true, Ordering::Release);
        self.handle.join();
    }
}