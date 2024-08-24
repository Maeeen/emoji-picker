use std::{cell::RefCell, env::Args, sync::mpsc::{self, Receiver, SyncSender}, thread::JoinHandle};

pub struct OnCloseHandler<'a, Args>(RefCell<Box<dyn FnMut(Args) + 'a>>);

impl<'a, Args> OnCloseHandler<'a, Args> {
    pub fn new<F>(f: F) -> Self
    where F: FnMut(Args) + 'a  {
        Self(RefCell::new(Box::new(f)))
    }

    pub fn call(&self, a: Args) {
        self.0.borrow_mut().as_mut()(a);
    }
}

pub struct OnOpenHandler<'a, Args>(RefCell<Box<dyn FnMut(Args) + 'a>>);

impl<'a, Args> OnOpenHandler<'a, Args> {
    pub fn new<F>(f: F) -> Self
    where F: FnMut(Args) + 'a {
        Self(RefCell::new(Box::new(f)))
    }

    pub fn call(&self, a: Args) {
        self.0.borrow_mut().as_mut()(a);
    }
}

pub trait Opener<Args> {
    fn has_requested_open(&self) -> Option<Args>;
}

// Basic opener
pub struct BasicOpener<Args = ()>(mpsc::Receiver<Args>);

impl<Args> BasicOpener<Args> {
    pub fn new(receiver: mpsc::Receiver<Args>) -> Self {
        Self(receiver)
    }
}

impl<Args> Opener<Args> for BasicOpener<Args> {
    fn has_requested_open(&self) -> Option<Args> {
        self.0.try_recv().ok()
    }
}

pub struct BasicCloser<Args = ()>(mpsc::Receiver<Args>);

pub trait Closer<Args = ()> {
    fn try_recv(&self) -> Option<Args>;
}

impl BasicCloser<Args> {
    pub fn new(receiver: mpsc::Receiver<Args>) -> Self {
        Self(receiver)
    }
}

impl Closer<Args> for BasicCloser<Args> {
    fn try_recv(&self) -> Option<Args> {
        self.0.try_recv().ok()
    }
}