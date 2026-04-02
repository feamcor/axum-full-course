//! # Module 10: Advanced Features
//!
//! `WebSockets`, SSE, File uploads, Static files

use axum::Router;
use axum::extract::Multipart;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::ws::WebSocketUpgrade;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::sse::Event;
use axum::response::sse::KeepAlive;
use axum::response::sse::Sse;
use axum::routing::get;
use axum::routing::post;
use futures::stream::Stream;
use futures::stream::{
    self,
};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;
use tower_http::services::ServeDir;

// ============================================================================
// LESSON 1: WebSocket
// ============================================================================

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        if let Ok(Message::Text(text)) = msg {
            let response = format!("Echo: {text}");
            if socket.send(Message::Text(response.into())).await.is_err() {
                break;
            }
        }
    }
}

// ============================================================================
// LESSON 2: Server-Sent Events (SSE)
// ============================================================================

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream =
        stream::repeat_with(|| Event::default().data(format!("Server time: {:?}", std::time::SystemTime::now())))
            .map(Ok)
            .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// ============================================================================
// LESSON 3: File Upload (Multipart)
// ============================================================================

async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let mut files = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.unwrap();
        files.push(format!("{}: {} bytes", name, data.len()));
    }

    if files.is_empty() {
        "No files uploaded".to_string()
    } else {
        format!("Uploaded: {}", files.join(", "))
    }
}

// ============================================================================
// LESSON 4: HTML Demo Page
// ============================================================================

async fn demo_page() -> Html<&'static str> {
    Html(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Axum Advanced Features</title>
    <style>
        body { font-family: system-ui; max-width: 800px; margin: 50px auto; padding: 20px; }
        .demo { background: #f5f5f5; padding: 20px; margin: 20px 0; border-radius: 8px; }
        button { padding: 10px 20px; margin: 5px; cursor: pointer; }
        #ws-output, #sse-output { height: 100px; overflow-y: auto; background: #fff; 
                                   border: 1px solid #ddd; padding: 10px; margin-top: 10px; }
    </style>
</head>
<body>
    <h1>🚀 Axum Advanced Features</h1>
    
    <div class="demo">
        <h2>WebSocket Echo</h2>
        <input type="text" id="ws-input" placeholder="Type message">
        <button onclick="sendWs()">Send</button>
        <div id="ws-output"></div>
    </div>
    
    <div class="demo">
        <h2>Server-Sent Events</h2>
        <button onclick="startSse()">Start SSE</button>
        <button onclick="stopSse()">Stop</button>
        <div id="sse-output"></div>
    </div>
    
    <div class="demo">
        <h2>File Upload</h2>
        <form action="/upload" method="post" enctype="multipart/form-data">
            <input type="file" name="file" multiple>
            <button type="submit">Upload</button>
        </form>
    </div>

    <script>
        let ws, sse;
        
        ws = new WebSocket('ws://localhost:3000/ws');
        ws.onmessage = (e) => {
            document.getElementById('ws-output').innerHTML += e.data + '<br>';
        };
        
        function sendWs() {
            ws.send(document.getElementById('ws-input').value);
        }
        
        function startSse() {
            sse = new EventSource('/sse');
            sse.onmessage = (e) => {
                document.getElementById('sse-output').innerHTML = e.data;
            };
        }
        
        function stopSse() { if(sse) sse.close(); }
    </script>
</body>
</html>
"#,
    )
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() {
    // Create static dir if needed
    std::fs::create_dir_all("static").ok();
    std::fs::write("static/hello.txt", "Hello from static file!").ok();

    let app = Router::new()
        .route("/", get(demo_page))
        .route("/ws", get(ws_handler))
        .route("/sse", get(sse_handler))
        .route("/upload", post(upload))
        .nest_service("/static", ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Module 10: Advanced Features");
    println!("   Server: http://localhost:3000\n");
    println!("📝 Features:");
    println!("   GET  /     - Demo page");
    println!("   WS   /ws   - WebSocket echo");
    println!("   GET  /sse  - Server-Sent Events");
    println!("   POST /upload - File upload");
    println!("   GET  /static/* - Static files");

    axum::serve(listener, app).await.unwrap();
}
