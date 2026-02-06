g::get, response::Html, Router};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // 1. Jalankan Bot Scanner di Background (Simulasi)
    tokio::spawn(async move {
        loop {
            println!("System: Scanning Solana Mempool..."); 
            sleep(Duration::from_secs(5)).await;
        }
    });

    // 2. Web Server Arkheion (Port 80 biar langsung akses domain/IP)
    let app = Router::new().route("/", get(handler));
    
    // Listen di semua interface (0.0.0.0) port 80
    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    println!("🚀 Arkheion System LIVE at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 3. Tampilan Halaman Depan (Landing Page)
async fn handler() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>ARKHEION | INTELLIGENCE</title>
        <style>
            body { background-color: #000; color: #0f0; font-family: monospace; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; }
            .container { text-align: center; border: 1px solid #333; padding: 2rem; box-shadow: 0 0 20px rgba(0, 255, 0, 0.2); }
            h1 { letter-spacing: 5px; text-transform: uppercase; }
            .status { color: #555; margin-top: 1rem; }
            .blink { animation: blinker 1s linear infinite; }
            @keyframes blinker { 50% { opacity: 0; } }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>Arkheion <span class="blink">_</span></h1>
            <p>SOLANA MEMPOOL MONITOR: <span style="color:cyan">ACTIVE</span></p>
            <p>VPS LOCATION: <span style="color:yellow">US-EAST (LOW LATENCY)</span></p>
            <div class="status">SYSTEM READY. WAITING FOR LIQUIDITY EVENTS.</div>
        </div>
    </body>
    </html>
    "#)
}
