use crate::{handler::Handler, SharedApp};

use super::BeforeOpenHandler;

/// This is to trigger a callback in Slint when the UI opens.
pub fn get_handler<'a>() -> BeforeOpenHandler<'a> {
    Handler::new(|args: &(SharedApp, _)| {
        let (app, _) = args;
        let _ = app
            .weak_ui()
            .upgrade_in_event_loop(|ui| ui.invoke_on_open());
    })
}
