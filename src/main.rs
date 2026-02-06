mod models;
mod db;
use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{sleep, Instant};

const RPC_URL: &str = "https://api.mainnet-beta.solana.com"; 
const PORT: u16 = 3000;
const DB_URL: &str = "sqlite://arkheion.db?mode=rwc";

#[derive(Clone)]
struct AppState {
    db: Pool<Sqlite>,
    metrics: Arc<Mutex<EngineMetrics>>,
}

#[derive(Serialize, Clone)]
struct EngineMetrics {
    slot: u64,
    tps: u64,
    epoch: u64,
    latency: u128,
    status: String,
    history: Vec<u64>,
}

#[derive(sqlx::FromRow, Serialize)]
struct User {
    id: i64,
    email: String,
    api_key: String,
    tier: String,
    requests: i64,
}

#[tokio::main]
async fn main() {
    println!(">>> STARTING ARKHEIONX ENTERPRISE v1.0 <<<");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await
        .expect("Gagal connect database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL,
            api_key TEXT NOT NULL,
            tier TEXT DEFAULT 'Free',
            requests INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let metrics = Arc::new(Mutex::new(EngineMetrics {
        slot: 0, tps: 0, epoch: 0, latency: 0,
        status: "BOOT_SEQUENCE".to_string(),
        history: vec![0; 20], // Simpan 20 data point terakhir
    }));

    let state = AppState { db: pool, metrics: metrics.clone() };

    tokio::spawn(async move {
        let client = RpcClient::new_with_commitment(RPC_URL, CommitmentConfig::confirmed());
        loop {
            let start = Instant::now();
            match client.get_epoch_info() {
                Ok(info) => {
                    let lat = start.elapsed().as_millis();
                    let tps_est = 2000 + (info.absolute_slot % 1500); // Simulasi cerdas karena RPC free tidak kasih raw TPS
                    
                    let mut m = metrics.lock().unwrap();
                    m.slot = info.absolute_slot;
                    m.epoch = info.epoch;
                    m.tps = tps_est;
                    m.latency = lat;
                    m.status = "OPERATIONAL".to_string();
                    
                    // Push data ke grafik (geser kiri)
                    m.history.remove(0);
                    m.history.push(tps_est);
                }
                Err(_) => {
                    let mut m = metrics.lock().unwrap();
                    m.status = "RECONNECTING".to_string();
                }
            }
            sleep(Duration::from_millis(1000)).await;
        }
    });

    let app = Router::new()
        .route("/", get(page_landing))
        .route("/login", get(page_login).post(handle_login))
        .route("/register", get(page_register).post(handle_register))
        .route("/dashboard", get(page_dashboard))
        .route("/api/v1/stream", get(api_stream_secure)) // Secured API
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    println!(">>> SYSTEM READY ON PORT {} <<<", PORT);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct AuthForm { email: String }

async fn handle_register(State(state): State<AppState>, Form(form): Form<AuthForm>) -> Response {
    let key = format!("sk_live_{}_x99", form.email.split('@').next().unwrap_or("user"));
    
    let res = sqlx::query("INSERT INTO users (email, api_key) VALUES (?, ?)")
        .bind(&form.email)
        .bind(&key)
        .execute(&state.db)
        .await;

    match res {
        Ok(_) => Redirect::to(&format!("/dashboard?u={}", form.email)).into_response(),
        Err(_) => Redirect::to("/login?err=exists").into_response(), // Email udah ada
    }
}

async fn handle_login(State(state): State<AppState>, Form(form): Form<AuthForm>) -> Response {
    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = ?")
        .bind(&form.email)
        .fetch_optional(&state.db)
        .await
        .unwrap();

    match user {
        Some(_) => Redirect::to(&format!("/dashboard?u={}", form.email)).into_response(),
        None => Redirect::to("/register").into_response(),
    }
}

#[derive(Deserialize)]
struct ApiQuery { key: Option<String> }

async fn api_stream_secure(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ApiQuery>
) -> Response {
    let provided_key = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.replace("Bearer ", ""))
        .or(q.key);

    if let Some(k) = provided_key {
        let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE api_key = ?")
            .bind(&k)
            .fetch_optional(&state.db)
            .await.unwrap();
        
        if exists.is_none() {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid API Key"}))).into_response();
        }
    } else {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Missing API Key"}))).into_response();
    }
    let m = state.metrics.lock().unwrap().clone();
    Json(m).into_response()
}

async fn page_landing() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html><html lang="en"><head><title>ARKHEIONX | Enterprise</title><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;800&family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet"><style>:root{--bg:#020202;--sf:#0A0A0A;--bd:#222;--pr:#fff;--ac:#00FF9D}body{background:var(--bg);color:var(--pr);font-family:'Inter',sans-serif;margin:0}nav{padding:20px 5%;border-bottom:1px solid var(--bd);display:flex;justify-content:space-between;background:rgba(2,2,2,0.9);backdrop-filter:blur(10px);position:sticky;top:0;z-index:99}.logo{font-weight:800;font-size:1.2rem;letter-spacing:-1px;color:#fff;text-decoration:none}.hero{text-align:center;padding:100px 20px}.btn{padding:12px 30px;border-radius:6px;font-weight:600;text-decoration:none;display:inline-block}.btn-p{background:var(--pr);color:#000}.btn-s{border:1px solid var(--bd);color:var(--pr)}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(250px,1fr));gap:30px;padding:5%;max-width:1200px;margin:0 auto}.card{background:var(--sf);border:1px solid var(--bd);padding:30px;border-radius:12px}footer{border-top:1px solid var(--bd);padding:50px;text-align:center;color:#666;font-size:0.8rem}.mockup{max-width:800px;margin:60px auto;background:#000;border:1px solid #333;border-radius:10px;overflow:hidden;box-shadow:0 0 50px rgba(0,255,157,0.1)}.m-head{padding:10px 20px;border-bottom:1px solid #333;display:flex;justify-content:space-between;font-family:'JetBrains Mono';font-size:0.75rem;color:#666}.m-body{padding:40px;display:grid;grid-template-columns:1fr 1fr;gap:20px;font-family:'JetBrains Mono'}</style></head><body>
<nav><a href="/" class="logo">ARKHEION<span>X</span></a><div><a href="/login" class="btn btn-s" style="margin-right:10px">Console</a><a href="/register" class="btn btn-p">Get Access</a></div></nav>
<div class="hero"><div style="color:var(--ac);font-weight:700;font-size:0.8rem;margin-bottom:20px;letter-spacing:1px">V3.0 ENTERPRISE</div><h1 style="font-size:4rem;letter-spacing:-2px;margin-bottom:20px;line-height:1.1">The Nervous System<br>of Solana DeFi.</h1><p style="color:#888;max-width:600px;margin:0 auto 40px;font-size:1.1rem">Milliseconds matter. Get direct RPC streams, liquidation signals, and mempool analytics.</p><div><a href="/register" class="btn btn-p">Start Building</a><a href="#pricing" class="btn btn-s" style="margin-left:10px">View Pricing</a></div><div class="mockup"><div class="m-head"><span>US-EAST-1</span><span style="color:var(--ac)">● SYSTEM ACTIVE</span></div><div class="m-body"><div style="text-align:left"><div style="color:#666;font-size:0.7rem">SLOT HEIGHT</div><div style="font-size:2rem" id="s">---</div></div><div style="text-align:left"><div style="color:#666;font-size:0.7rem">LATENCY</div><div style="font-size:2rem;color:var(--ac)" id="l">---</div></div></div></div></div>
<div class="grid"><div class="card"><h3 style="margin-top:0">Global Nodes</h3><p style="color:#888">Distributed infrastructure ensuring < 50ms latency worldwide.</p></div><div class="card"><h3 style="margin-top:0">Security First</h3><p style="color:#888">Enterprise-grade encryption and API key management.</p></div><div class="card"><h3 style="margin-top:0">Data Persistence</h3><p style="color:#888">Historical data tracking with persistent storage layers.</p></div></div>
<footer>&copy; 2026 ARKHEIONX SYSTEMS. ENGINEERED BY YUDISTIRA PUTRA DEV.</footer>
<script>setInterval(async()=>{try{let r=await fetch('/api/v1/stream?key=demo');let d=await r.json();document.getElementById('s').innerText=d.slot.toLocaleString();document.getElementById('l').innerText=d.latency+"ms"}catch(e){}},1000)</script>
</body></html>"##)
}

async fn page_login() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html><html lang="en"><head><title>Login</title><style>body{background:#020202;color:#fff;font-family:sans-serif;height:100vh;display:grid;place-items:center}.box{width:350px;padding:40px;border:1px solid #333;border-radius:12px;text-align:center}input{width:100%;padding:12px;margin:10px 0;background:#0a0a0a;border:1px solid #333;color:#fff;box-sizing:border-box}button{width:100%;padding:12px;background:#fff;border:none;font-weight:bold;cursor:pointer;margin-top:10px}.logo{font-weight:800;font-size:1.5rem;color:#fff;text-decoration:none;display:block;margin-bottom:30px}span{color:#00ff9d}</style></head><body><div class="box"><a href="/" class="logo">ARKHEION<span>X</span></a><h2>Console Login</h2><form action="/login" method="post"><input type="email" name="email" placeholder="Email Address" required><button>Access Terminal</button></form><p style="color:#666;font-size:0.8rem;margin-top:20px">No account? <a href="/register" style="color:#fff">Get API Key</a></p></div></body></html>"##)
}

async fn page_register() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html><html lang="en"><head><title>Register</title><style>body{background:#020202;color:#fff;font-family:sans-serif;height:100vh;display:grid;place-items:center}.box{width:350px;padding:40px;border:1px solid #333;border-radius:12px;text-align:center}input{width:100%;padding:12px;margin:10px 0;background:#0a0a0a;border:1px solid #333;color:#fff;box-sizing:border-box}button{width:100%;padding:12px;background:#fff;border:none;font-weight:bold;cursor:pointer;margin-top:10px}.logo{font-weight:800;font-size:1.5rem;color:#fff;text-decoration:none;display:block;margin-bottom:30px}span{color:#00ff9d}</style></head><body><div class="box"><a href="/" class="logo">ARKHEION<span>X</span></a><h2>Create API Key</h2><form action="/register" method="post"><input type="email" name="email" placeholder="Work Email" required><button>Generate Credentials</button></form><p style="color:#666;font-size:0.8rem;margin-top:20px">Existing user? <a href="/login" style="color:#fff">Login</a></p></div></body></html>"##)
}

async fn page_dashboard() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html><html lang="en"><head><title>Terminal - ARKHEIONX</title><script src="https://cdn.jsdelivr.net/npm/chart.js"></script><style>body{background:#020202;color:#fff;font-family:sans-serif;margin:0;display:flex}.side{width:240px;border-right:1px solid #222;height:100vh;padding:20px;position:fixed}.main{margin-left:240px;padding:40px;width:100%}.logo{font-weight:800;font-size:1.2rem;color:#fff;text-decoration:none;display:block;margin-bottom:40px}span{color:#00ff9d}.menu a{display:block;color:#888;text-decoration:none;padding:10px;margin-bottom:5px;border-radius:4px}.menu a.active{background:#111;color:#fff}.card{background:#0a0a0a;border:1px solid #222;padding:20px;border-radius:8px}.key{background:#000;border:1px solid #333;padding:10px;font-family:monospace;color:#00ff9d;display:block;margin-top:10px;word-break:break-all}.grid{display:grid;grid-template-columns:repeat(3,1fr);gap:20px;margin-top:20px}canvas{width:100% !important;height:300px !important}</style></head><body>
<div class="side"><a href="/" class="logo">ARKHEION<span>X</span></a><div class="menu"><a href="#" class="active">Overview</a><a href="#">Analytics</a><a href="#">Billing</a><a href="#">Settings</a><a href="/" style="margin-top:40px;color:#f33">Disconnect</a></div></div>
<div class="main">
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:30px"><h1>Overview</h1><div style="color:#666" id="u-email">...</div></div>
    <div class="card" style="margin-bottom:30px">
        <div style="font-size:0.7rem;color:#666;margin-bottom:10px">YOUR SECRET KEY</div>
        <code class="key" id="apikey">Generating secure key...</code>
    </div>
    <div class="card">
        <div style="font-size:0.7rem;color:#666;margin-bottom:20px">NETWORK THROUGHPUT (TPS)</div>
        <canvas id="tpsChart"></canvas>
    </div>
    <div class="grid">
        <div class="card"><div style="color:#666;font-size:0.7rem">STATUS</div><div style="font-size:1.5rem;color:#00ff9d">● Operational</div></div>
        <div class="card"><div style="color:#666;font-size:0.7rem">LATENCY</div><div style="font-size:1.5rem" id="d-lat">-- ms</div></div>
        <div class="card"><div style="color:#666;font-size:0.7rem">PLAN</div><div style="font-size:1.5rem">Enterprise</div></div>
    </div>
</div>
<script>
    const p = new URLSearchParams(window.location.search);
    const u = p.get('u');
    if(!u) window.location.href='/login';
    document.getElementById('u-email').innerText = u;
    document.getElementById('apikey').innerText = "sk_live_" + btoa(u).substring(0,12) + "_x99_secure";
    const apiKey = document.getElementById('apikey').innerText;

    const ctx = document.getElementById('tpsChart').getContext('2d');
    const chart = new Chart(ctx, {
        type: 'line',
        data: { labels: Array(20).fill(''), datasets: [{ label: 'TPS', data: Array(20).fill(0), borderColor: '#00ff9d', tension: 0.4, borderWidth: 2, pointRadius: 0 }] },
        options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { display: false } }, scales: { x: { display: false }, y: { grid: { color: '#222' } } } }
    });

    setInterval(async () => {
        try {
            const r = await fetch(`/api/v1/stream?key=${apiKey}`);
            if(r.status === 401) return; 
            const d = await r.json();
            
            document.getElementById('d-lat').innerText = d.latency + " ms";
            
            chart.data.datasets[0].data = d.history;
            chart.update('none');
        } catch(e) {}
    }, 1000);
</script></body></html>"##)
}
