use std::{rc::Rc, sync::{mpsc, Mutex}};

/// This represents a very basic handler that can be used to handle events.
/// It is very important that the handler does not lock anything.
/// To get more into details: the handler has a Mutex so that the handler
/// can be `Sync`. So, it is really important that a handler does not
/// try to execute any other handler (which is a necessary but not sufficient
/// condition) to avoid dead-locks.
// #[derive(Clone)]
pub struct Handler<'a, Args>(pub Mutex<Box<dyn Fn(&Args) + Send + 'a>>);

impl<'a, Args> Handler<'a, Args> {
    /// Creates a new handler with the given closure.
    pub fn new<F>(f: F) -> Self
    where F: Fn(&Args) + Send + 'a  {
        Self(Mutex::new(Box::new(f)))
    }

    /// Calls the handler with the given arguments.
    pub fn call<'b>(&self, a: &'b Args) {
        self.0.lock().unwrap()(a);
    }
}

/// A notifier. Technically, it is the same as a handler but in the reverse way.
/// In principle, it is useless. But, it can be used to make the code more readable.
/// (ie, the personal preference of callback-hell vs event-driven programming)
pub trait Notifier<Args> {
    fn has_notified(&self) -> Option<Args>;
}

/// A notifier can be a simple `mpsc::Receiver`.
impl<Args> Notifier<Args> for mpsc::Receiver<Args> {
    fn has_notified(&self) -> Option<Args> {
        self.try_recv().ok()
    }
}