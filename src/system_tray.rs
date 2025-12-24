use std::sync::{Arc, Mutex};
use crate::AppState;

// Windows system tray icon with quit option
pub fn run(state: Arc<Mutex<AppState>>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        use tray_item::{TrayItem, IconSource};

        if let Ok(mut tray) = TrayItem::new("MITB-AddressHijacker", IconSource::Resource("app")) {
            let _ = tray.add_label("Active");

            let quit_state = Arc::clone(&state);
            let _ = tray.add_menu_item("Quit", move || {
                quit_state.lock().unwrap().running = false;
                std::process::exit(0);
            });
        }
    }

    // Keep thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if !state.lock().unwrap().running {
            break;
        }
    }
    Ok(())
}
