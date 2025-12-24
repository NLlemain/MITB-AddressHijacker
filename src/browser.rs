use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;
use serde::Deserialize;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::AppState;
use crate::injector;

const CDP_PORTS: &[u16] = &[9222, 9223, 9224];

#[derive(Debug, Deserialize, Clone)]
pub struct CDPTarget {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "type")]
    pub target_type: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub ws_url: Option<String>,
}

pub struct BrowserManager {
    cdp_port: Option<u16>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self { cdp_port: None }
    }
    
    pub async fn connect(&mut self) -> Option<Vec<CDPTarget>> {
        if let Some(port) = self.cdp_port {
            let url = format!("http://127.0.0.1:{}/json", port);
            if let Ok(resp) = reqwest::get(&url).await {
                if let Ok(targets) = resp.json::<Vec<CDPTarget>>().await {
                    return Some(targets);
                }
            }
            self.cdp_port = None;
        }
        
        for port in CDP_PORTS {
            let url = format!("http://127.0.0.1:{}/json", port);
            if let Ok(resp) = reqwest::get(&url).await {
                if let Ok(targets) = resp.json::<Vec<CDPTarget>>().await {
                    self.cdp_port = Some(*port);
                    return Some(targets);
                }
            }
        }
        None
    }
    
    pub async fn inject(&self, target: &CDPTarget) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ws_url) = &target.ws_url {
            injector::inject(ws_url, injector::get_script()).await?;
        }
        Ok(())
    }
    
    #[cfg(windows)]
    pub fn restart_edge() -> bool {
        use std::process::Command;
        
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "msedge.exe"])
            .creation_flags(0x08000000)
            .output();
        
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        let paths = [
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        ];
        
        for path in paths {
            if std::path::Path::new(path).exists() {
                if Command::new(path).arg("--remote-debugging-port=9222").spawn().is_ok() {
                    return true;
                }
            }
        }
        false
    }
}

pub async fn monitor(state: Arc<Mutex<AppState>>) {
    let mut manager = BrowserManager::new();
    let mut injected: Vec<String> = Vec::new();
    let mut restart_tried = false;
    let mut connected = false;
    
    time::sleep(Duration::from_millis(500)).await;
    
    loop {
        if !state.lock().unwrap().running { break; }
        
        if let Some(targets) = manager.connect().await {
            connected = true;
            
            for target in &targets {
                if target.target_type == "page" && !injected.contains(&target.id) {
                    if target.url.starts_with("http") {
                        if manager.inject(target).await.is_ok() {
                            injected.push(target.id.clone());
                            state.lock().unwrap().pages_injected += 1;
                        }
                    }
                }
            }
            
            let ids: Vec<String> = targets.iter().map(|t| t.id.clone()).collect();
            injected.retain(|id| ids.contains(id));
            
        } else if !restart_tried && !connected {
            restart_tried = true;
            #[cfg(windows)]
            if BrowserManager::restart_edge() {
                time::sleep(Duration::from_secs(4)).await;
            }
        }
        
        time::sleep(Duration::from_millis(if connected { 500 } else { 2000 })).await;
    }
}
