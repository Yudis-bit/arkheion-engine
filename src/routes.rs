use axum::{
    extract::{State, Query},
    response::{Html, IntoResponse, Json},
};
use std::sync::Arc;
use serde_json::json;
use crate::models::AppState;

pub async fn get_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics.lock().unwrap();
    
    Json(json!({
        "network": "solana-mainnet",
        "data": {
            "slot": metrics.slot,
            "tps": metrics.tps,
            "epoch": metrics.epoch,
            "latency_ms": metrics.latency,
            "status": metrics.status
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn landing_page() -> Html<&'static str> {
    Html(r##"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ARKHEIONX | High-Frequency Intelligence</title>
    <style>
        :root { --bg: #050505; --card: #0F0F0F; --accent: #00FF9D; --text: #EAEAEA; --dim: #666; }
        body { background: var(--bg); color: var(--text); font-family: 'Courier New', monospace; margin: 0; display: flex; flex-direction: column; min-height: 100vh; }
        .nav { padding: 20px 40px; border-bottom: 1px solid #222; display: flex; justify-content: space-between; align-items: center; background: rgba(5,5,5,0.9); backdrop-filter: blur(10px); position: sticky; top: 0; }
        .logo { font-weight: bold; font-size: 1.5rem; color: #fff; letter-spacing: -1px; text-decoration: none; }
        .logo span { color: var(--accent); }
        .hero { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; padding: 60px 20px; }
        h1 { font-size: 3.5rem; line-height: 1; margin-bottom: 20px; letter-spacing: -2px; }
        .btn { padding: 12px 30px; font-weight: bold; text-decoration: none; border-radius: 4px; transition: 0.2s; cursor: pointer; border: none; font-family: inherit; }
        .btn-glow { background: var(--accent); color: #000; box-shadow: 0 0 20px rgba(0, 255, 157, 0.2); }
        .btn-glow:hover { box-shadow: 0 0 40px rgba(0, 255, 157, 0.5); transform: scale(1.05); }
        .terminal { background: var(--card); border: 1px solid #333; padding: 20px; border-radius: 8px; margin-top: 50px; width: 100%; max-width: 600px; text-align: left; position: relative; overflow: hidden; }
        .terminal::before { content: ''; position: absolute; top: 0; left: 0; width: 100%; height: 2px; background: linear-gradient(90deg, transparent, var(--accent), transparent); animation: scan 2s infinite; }
        @keyframes scan { 0% { left: -100%; } 100% { left: 100%; } }
        .stat-row { display: flex; justify-content: space-between; margin-bottom: 10px; font-size: 0.9rem; }
        .val { color: var(--accent); font-weight: bold; }
    </style>
</head>
<body>
    <nav>
        <a href="/" class="logo">ARKHEION<span>X</span></a>
        <div>
            <a href="/login" style="color: #fff; text-decoration: none; margin-right: 20px;">Console</a>
            <a href="/dashboard" class="btn btn-glow">Connect Wallet</a>
        </div>
    </nav>
    
    <div class="hero">
        <div style="color: var(--dim); margin-bottom: 20px;">V3.0 // MAINNET BETA</div>
        <h1>THE NERVOUS SYSTEM<br>OF SOLANA DEFI</h1>
        <p style="color: var(--dim); max-width: 500px; margin-bottom: 40px;">
            Institutional-grade RPC intelligence. Detect liquidations, whale movements, and mempool opportunities in milliseconds.
        </p>
        
        <div class="terminal">
            <div style="border-bottom: 1px solid #333; padding-bottom: 10px; margin-bottom: 15px; display: flex; justify-content: space-between;">
                <span>LIVE FEED // US-EAST</span>
                <span style="color: var(--accent);">● ONLINE</span>
            </div>
            <div class="stat-row"><span>SLOT HEIGHT</span><span class="val" id="s_slot">---</span></div>
            <div class="stat-row"><span>TPS (AVG)</span><span class="val" id="s_tps">---</span></div>
            <div class="stat-row"><span>EPOCH</span><span class="val" id="s_epoch">---</span></div>
            <div class="stat-row"><span>LATENCY</span><span class="val" id="s_lat">---</span></div>
        </div>
    </div>

    <script>
        async function sync() {
            try {
                let res = await fetch('/api/metrics');
                let d = await res.json();
                document.getElementById('s_slot').innerText = parseInt(d.data.slot).toLocaleString();
                document.getElementById('s_tps').innerText = d.data.tps;
                document.getElementById('s_epoch').innerText = "#" + d.data.epoch;
                document.getElementById('s_lat').innerText = d.data.latency_ms + "ms";
            } catch(e) {}
        }
        setInterval(sync, 1000); // Update setiap 1 detik
        sync();
    </script>
</body>
</html>
    "##)
}

pub async fn dashboard_page() -> Html<&'static str> {
    Html(r##"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Dashboard | ARKHEIONX</title>
    <style>
        body { background: #050505; color: #fff; font-family: sans-serif; display: flex; height: 100vh; margin: 0; }
        .sidebar { width: 250px; border-right: 1px solid #222; padding: 20px; display: flex; flex-direction: column; }
        .content { flex: 1; padding: 40px; }
        .logo { font-size: 1.2rem; font-weight: 800; margin-bottom: 40px; color: #fff; text-decoration: none; }
        .menu-item { padding: 12px; color: #888; text-decoration: none; display: block; border-radius: 6px; margin-bottom: 5px; }
        .menu-item:hover, .menu-item.active { background: #111; color: #fff; }
        .card { background: #0F0F0F; border: 1px solid #222; padding: 25px; border-radius: 10px; margin-bottom: 20px; }
        .key-box { background: #000; border: 1px solid #333; padding: 15px; font-family: monospace; color: #00FF9D; margin-top: 10px; word-break: break-all; }
        h2 { margin-top: 0; font-size: 1.5rem; }
        .badge { background: rgba(0,255,157,0.1); color: #00FF9D; padding: 5px 10px; border-radius: 20px; font-size: 0.8rem; border: 1px solid rgba(0,255,157,0.2); }
    </style>
</head>
<body>
    <div class="sidebar">
        <a href="/" class="logo">ARKHEION<span style="color:#00FF9D">X</span></a>
        <a href="#" class="menu-item active">Overview</a>
        <a href="#" class="menu-item">API Keys</a>
        <a href="#" class="menu-item">Billing (Web3)</a>
        <a href="#" class="menu-item">Documentation</a>
        <div style="margin-top: auto;">
            <a href="/" class="menu-item" style="color: #f55;">Disconnect</a>
        </div>
    </div>
    <div class="content">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:30px;">
            <h2>Console Overview</h2>
            <div class="badge">TIER: FREE</div>
        </div>
        
        <div class="card">
            <div style="color: #666; font-size: 0.8rem; margin-bottom: 10px;">YOUR API KEY</div>
            <div class="key-box">sk_live_883a9f_demo_key_x99</div>
            <div style="color: #444; font-size: 0.8rem; margin-top: 10px;">Use this key to authenticate your requests. Limit: 5 req/sec.</div>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
            <div class="card">
                <div style="color: #666; font-size: 0.8rem;">REQUESTS TODAY</div>
                <div style="font-size: 2rem; margin-top: 10px;">1,024 <span style="font-size:1rem; color:#444;">/ 10,000</span></div>
            </div>
            <div class="card">
                <div style="color: #666; font-size: 0.8rem;">SERVICE STATUS</div>
                <div style="font-size: 2rem; margin-top: 10px; color: #00FF9D;">● Optimal</div>
            </div>
        </div>
    </div>
</body>
</html>
    "##)
}
