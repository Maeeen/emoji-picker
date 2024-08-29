use windows::Win32::Foundation::HWND;
use std::{fmt::Debug, sync::Arc};

mod backend_iced;
use backend_iced::BackendLinkInner;

type Listener = Arc<dyn Fn(&BackendLink, BackendEvents) + Send + Sync + 'static>;


pub struct BackendLink {
    inner: Arc<BackendLinkInner>
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum BackendEventKind {
    EmojiSelected,
    EmojiFilter,
    CloseRequested,
    OpenRequested,
}

pub enum BackendEvents {
    EmojiSelected(String),
    EmojiFilter(String),
    CloseRequested,
    OpenRequested,
}

impl BackendEvents {
    pub fn kind(&self) -> BackendEventKind {
        match self {
            BackendEvents::EmojiSelected(_) => BackendEventKind::EmojiSelected,
            BackendEvents::EmojiFilter(_) => BackendEventKind::EmojiFilter,
            BackendEvents::CloseRequested => BackendEventKind::CloseRequested,
            BackendEvents::OpenRequested => BackendEventKind::OpenRequested,
        }
    }
}

#[derive(Debug)]
enum BackendError {
    Invalid
}

impl BackendLink {
    pub fn new() -> Result<Arc<Self>, BackendError> {
        Ok(Arc::new(Self {
            inner: Arc::new(BackendLinkInner::new())
        }))
    }

    pub fn show(&self) {
        self.inner.show();
    }

    pub fn hide(&self) {
        self.inner.hide();
    }

    pub fn run_event_loop(&self) {
        self.inner.run_event_loop();
    }

    pub fn get_main_window_hwnd(&self) -> Option<HWND> {
        Some(HWND(0 as *mut _))
    }

    pub fn set_position<T: Debug>(&self, position: T) {
        println!("Setting position: {:?}", position);
    }

    pub fn quit(&self) {
        std::process::exit(0);
    }

    pub fn on(&self, event: BackendEventKind, callback: Listener) {
        self.inner.on(event, callback);
    }

    pub fn dispatch(&self, event: BackendEvents) {
        self.inner.dispatch(&self, event);
    }
}