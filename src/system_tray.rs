use std::sync::{Arc, Mutex};
use crate::AppState;

pub fn run(state: Arc<Mutex<AppState>>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        use tray_item::{TrayItem, IconSource};
        
        if let Ok(mut tray) = TrayItem::new("Crypto Clipper", IconSource::Resource("app")) {
            let _ = tray.add_label("Crypto Clipper Active");
            
            let quit_state = Arc::clone(&state);
            let _ = tray.add_menu_item("Quit", move || {
                quit_state.lock().unwrap().running = false;
                std::process::exit(0);
            });
        }
    }
    
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if !state.lock().unwrap().running { break; }
    }
    Ok(())
}
