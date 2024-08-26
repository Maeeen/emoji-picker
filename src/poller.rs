use core::time;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct Poller {
    semaphore: Arc<AtomicBool>,
}

impl Poller {
    pub fn new(mut f: impl FnMut() + Send + 'static) -> Self {
        let arc = Arc::new(AtomicBool::new(false));
        let arc_cloned = arc.clone();
        // This is not nice. Might switch to slint::spawn
        std::thread::spawn(move || loop {
            if arc_cloned.load(Ordering::Acquire) {
                break;
            }
            f();
            std::thread::sleep(time::Duration::from_millis(100));
        });

        Poller { semaphore: arc }
    }

    pub fn signal_stop(&self) {
        self.semaphore.store(true, Ordering::Release);
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        self.signal_stop();
    }
}
