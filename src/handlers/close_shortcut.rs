use std::sync::Arc;

use crate::{backend_link::BackendLink, handler::{MpscNotifier, Notifier}};

pub fn get_close_shortcut_notifier(app: &BackendLink) -> Box<dyn Notifier<()>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    app.on(crate::backend_link::BackendEventKind::CloseRequested, Arc::new(move |_, _| {
        tx.send(()).unwrap();
    }));
    Box::new(MpscNotifier::new(rx))
}
