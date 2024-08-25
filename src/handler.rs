use std::sync::{mpsc, Mutex};

/// This represents a very basic handler that can be used to handle events.
/// It is very important that the handler does not lock anything.
/// To get more into details: the handler has a Mutex so that the handler
/// can be `Sync`. So, it is really important that a handler does not
/// try to execute any other handler (which is a necessary but not sufficient
/// condition) to avoid dead-locks.
pub struct Handler<'a, Args>(pub Mutex<Box<dyn Fn(&Args) + Send + 'a>>);

impl<'a, Args> Handler<'a, Args> {
    /// Creates a new handler with the given closure.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&Args) + Send + 'a,
    {
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
pub trait Notifier<Args>: Send + Sync {
    fn has_notified(&self) -> Option<Args>;
}

/// A notifier can be a simple `mpsc::Receiver`. However, it requires a Mutex
/// to be `Sync` (which is required by the `Notifier` trait.)
pub struct MpscNotifier<Args>(Mutex<mpsc::Receiver<Args>>);

impl<Args> MpscNotifier<Args> {
    pub fn new(rx: mpsc::Receiver<Args>) -> Self {
        Self(Mutex::new(rx))
    }
}

impl<Args> Notifier<Args> for MpscNotifier<Args>
where
    Args: Send,
{
    fn has_notified(&self) -> Option<Args> {
        self.0.lock().unwrap().try_recv().ok()
    }
}

/// A notifier that notifies only once.
pub struct OnceNotifier<Args>(Mutex<Option<Args>>);

impl<Args> OnceNotifier<Args> {
    pub fn new(t: Args) -> Self {
        Self(Mutex::new(Some(t)))
    }
}

impl<Args> Default for OnceNotifier<Args>
where
    Args: Default,
{
    fn default() -> Self {
        Self(Mutex::new(Some(Default::default())))
    }
}

impl<Args> Notifier<Args> for OnceNotifier<Args>
where
    Args: Send,
{
    fn has_notified(&self) -> Option<Args> {
        self.0.lock().unwrap().take()
    }
}
