use std::sync::mpsc;
use tray_item::TrayItem;

use crate::handler::{MpscNotifier, Notifier};

use super::{NotifiersArgs, OpenerNotifier};

struct TrayIconNotifier {
    np: MpscNotifier<NotifiersArgs>,
    // If not there, the tray icon will be destroyed.
    #[allow(dead_code)]
    t: TrayItem, // To prevent the tray icon from being dropped.
}

/// This returns the notifier that opens.
pub fn initialize() -> OpenerNotifier {
    let (tx, rx) = mpsc::sync_channel::<NotifiersArgs>(1);
    let mut t =
        TrayItem::new("Emoji picker", tray_item::IconSource::Resource("tray-icon")).unwrap();

    // TODO: do something else than unwrap.
    t.add_menu_item("Show", move || {
        tx.send(super::NotifierReason::TrayIcon).unwrap();
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

impl Notifier<NotifiersArgs> for TrayIconNotifier {
    fn has_notified(&self) -> Option<NotifiersArgs> {
        self.np.has_notified()
    }
}
