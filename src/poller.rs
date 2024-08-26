use core::time;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

pub struct Poller {
    handle: JoinHandle<()>,
    semaphore: Arc<AtomicBool>,
}

impl Poller {
    pub fn new(mut f: impl FnMut() + Send + 'static) -> Self {
        let arc = Arc::new(AtomicBool::new(false));
        let arc_cloned = arc.clone();
        // This is not nice. Might switch to slint::spawn
        Poller {
            handle: std::thread::spawn(move || loop {
                if arc_cloned.load(Ordering::Acquire) {
                    break;
                }
                f();
                std::thread::sleep(time::Duration::from_millis(100));
            }),
            semaphore: arc,
        }
    }

    pub fn join(self) {
        self.semaphore.store(true, Ordering::Release);
        if let Err(e) = self.handle.join() {
            std::panic::resume_unwind(e)
        }
    }
}
