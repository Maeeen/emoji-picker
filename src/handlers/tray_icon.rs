use std::sync::mpsc;
use tray_item::TrayItem;

use crate::handler::{MpscNotifier, Notifier};

struct TrayIconNotifier {
    np: MpscNotifier<()>,
    // If not there, the tray icon will be destroyed.
    #[allow(dead_code)]
    t: TrayItem, // See if lower bounds are sufficient
}

/// This returns the open notifier.
pub fn initialize() -> Box<dyn Notifier<()>> {
    let (tx, rx) = mpsc::sync_channel::<()>(1);
    let mut t =
        TrayItem::new("Emoji picker", tray_item::IconSource::Resource("tray-icon")).unwrap();

    t.add_label("Emoji picker").unwrap();

    // TODO: do something else than unwrap.
    t.add_menu_item("Show", move || {
        tx.send(()).unwrap();
    })
    .unwrap();

    t.add_menu_item("Quit", move || {
        slint::quit_event_loop().unwrap();
    })
    .unwrap();

    Box::new(TrayIconNotifier {
        np: MpscNotifier::new(rx),
        t,
    })
}

impl Notifier<()> for TrayIconNotifier {
    fn has_notified(&self) -> Option<()> {
        self.np.has_notified()
    }
}
