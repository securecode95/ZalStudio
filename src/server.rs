use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread;

const UPLOAD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ZalStudio Upload</title>
<style>
  * { box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #0f0f18;
    color: #eee;
    margin: 0;
    padding: 24px;
    text-align: center;
  }
  h1 { color: #00c8ff; margin-top: 8px; font-size: 28px; }
  p { color: #aaa; font-size: 16px; margin: 8px 0 24px; }
  input[type=file] { display: none; }
  .btn {
    display: inline-block;
    background: #00b86b;
    color: #fff;
    padding: 18px 36px;
    border-radius: 14px;
    font-size: 18px;
    font-weight: bold;
    cursor: pointer;
    border: none;
    margin: 8px;
    width: 90%;
    max-width: 400px;
  }
  .btn:active { transform: scale(0.98); }
  #thumbs { margin-top: 20px; }
  .thumb {
    display: inline-block;
    width: 72px;
    height: 72px;
    object-fit: cover;
    margin: 4px;
    border-radius: 10px;
    border: 2px solid #333;
  }
  #status { margin-top: 16px; font-size: 15px; color: #888; min-height: 24px; }
  .done { color: #00e676; }
</style>
</head>
<body>
<h1>📷 ZalStudio</h1>
<p>Select photos to send to the kiosk</p>
<label class="btn" for="file">Choose Photos</label>
<input type="file" id="file" multiple accept="image/*">
<div id="thumbs"></div>
<div id="status"></div>
<script>
const fileInput = document.getElementById('file');
const status = document.getElementById('status');
const thumbs = document.getElementById('thumbs');

fileInput.onchange = async () => {
  const files = Array.from(fileInput.files);
  if (files.length === 0) return;
  status.textContent = 'Uploading ' + files.length + ' photo(s)...';
  status.className = '';
  let done = 0;
  for (const file of files) {
    const reader = new FileReader();
    await new Promise(resolve => {
      reader.onload = async () => {
        const base64 = reader.result.split(',')[1];
        try {
          const res = await fetch('/upload', {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({name: file.name, data: base64})
          });
          if (res.ok) {
            const img = document.createElement('img');
            img.src = reader.result;
            img.className = 'thumb';
            thumbs.appendChild(img);
            done++;
          }
        } catch (e) {
          console.error(e);
        }
        resolve();
      };
      reader.readAsDataURL(file);
    });
  }
  status.textContent = 'Done! ' + done + ' photo(s) sent to kiosk.';
  status.className = 'done';
};
</script>
</body>
</html>"#;

const AUTH_SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ZalStudio - Authorized</title>
<style>
  * { box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #0f0f18;
    color: #eee;
    margin: 0;
    padding: 40px 24px;
    text-align: center;
  }
  h1 { color: #00e676; font-size: 32px; margin-bottom: 16px; }
  p { color: #aaa; font-size: 18px; }
  .check { font-size: 64px; margin-bottom: 8px; }
</style>
</head>
<body>
<div class="check">✅</div>
<h1>Authorization complete!</h1>
<p>You can return to the kiosk.<br>Your Google Photos will appear shortly.</p>
</body>
</html>"#;

// Global store for the OAuth authorization code received from Google callback.
static AUTH_CODE_STORE: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();

fn auth_store() -> &'static Mutex<Option<(String, String)>> {
    AUTH_CODE_STORE.get_or_init(|| Mutex::new(None))
}

/// Take the stored auth code (code, state) if present, clearing it atomically.
pub fn take_auth_code() -> Option<(String, String)> {
    auth_store().lock().ok()?.take()
}

/// Peek whether an auth code has been stored.
pub fn has_auth_code() -> bool {
    auth_store()
        .lock()
        .ok()
        .map(|g| g.is_some())
        .unwrap_or(false)
}

pub fn start_server(upload_dir: PathBuf, port: u16) -> Option<String> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).ok()?;
    let local_addr = listener.local_addr().ok()?;
    let url = format!("http://{}:{}", local_ip(), local_addr.port());

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let dir = upload_dir.clone();
                thread::spawn(move || handle_client(stream, dir));
            }
        }
    });

    Some(url)
}

fn handle_auth_callback(path: &str, stream: &mut TcpStream) {
    let query = path.split('?').nth(1).unwrap_or("");
    let params: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .collect();

    if let (Some(code), Some(state)) = (params.get("code"), params.get("state")) {
        if let Ok(mut store) = auth_store().lock() {
            *store = Some((code.to_string(), state.to_string()));
        }
    }

    let body = AUTH_SUCCESS_HTML.as_bytes();
    let response = format!(
        "HTTP/1.1 200 OK\r\
        Content-Type: text/html; charset=utf-8\r\
        Content-Length: {}\r\
        Connection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(body);
}

fn handle_client(mut stream: TcpStream, upload_dir: PathBuf) {
    let mut reader = BufReader::new(&stream);
    let mut first_line = String::new();
    if reader.read_line(&mut first_line).is_err() {
        return;
    }

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let path = parts[1];

    // Read headers
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }

    // CORS preflight
    if method == "OPTIONS" {
        let response = "HTTP/1.1 204 No Content\r\
            Access-Control-Allow-Origin: *\r\
            Access-Control-Allow-Methods: POST, GET, OPTIONS\r\
            Access-Control-Allow-Headers: Content-Type\r\
            Connection: close\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    if method == "GET" && path == "/" {
        let body = UPLOAD_HTML.as_bytes();
        let response = format!(
            "HTTP/1.1 200 OK\r\
            Content-Type: text/html; charset=utf-8\r\
            Content-Length: {}\r\
            Access-Control-Allow-Origin: *\r\
            Connection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(body);
    } else if method == "GET" && path.starts_with("/auth/callback") {
        handle_auth_callback(&path, &mut stream);
    } else if method == "GET" && path.starts_with("/") {
        // Check if this is an OAuth callback (has ?code=...&state=...)
        if path.contains("?code=") || path.contains("&code=") {
            handle_auth_callback(&path, &mut stream);
        } else {
            let body = UPLOAD_HTML.as_bytes();
            let response = format!(
                "HTTP/1.1 200 OK\r\
                Content-Type: text/html; charset=utf-8\r\
                Content-Length: {}\r\
                Access-Control-Allow-Origin: *\r\
                Connection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(body);
        }
    } else if method == "POST" && path == "/upload" {
        let len = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let mut body = vec![0u8; len];
        if reader.read_exact(&mut body).is_err() {
            return;
        }

        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
            if let (Some(name), Some(data)) = (
                json.get("name").and_then(|v| v.as_str()),
                json.get("data").and_then(|v| v.as_str()),
            ) {
                if let Ok(bytes) = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    data,
                ) {
                    let _ = std::fs::create_dir_all(&upload_dir);
                    let safe_name = sanitize_filename(name);
                    let dest = upload_dir.join(&safe_name);
                    let _ = std::fs::write(dest, bytes);
                }
            }
        }

        let response = "HTTP/1.1 200 OK\r\
            Content-Length: 2\r\
            Access-Control-Allow-Origin: *\r\
            Connection: close\r\n\r\nOK";
        let _ = stream.write_all(response.as_bytes());
    } else {
        let response = "HTTP/1.1 404 Not Found\r\
            Content-Length: 0\r\
            Connection: close\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
    }
}

fn local_ip() -> String {
    // On Windows, the Mobile Hotspot interface typically uses 192.168.137.1.
    // Check if this address is actually bound to a local interface before
    // returning it, so we don't lie when hotspot is off.
    #[cfg(windows)]
    {
        if is_ip_local("192.168.137.1") {
            return "192.168.137.1".to_string();
        }
    }

    // Trick: connect a UDP socket to a non-routable address.
    // This forces the OS to pick an interface without sending any packets.
    // Works even without internet (offline hotspot mode).
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("10.255.255.255:1").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "0.0.0.0" && !ip.starts_with("127.") {
                    return ip;
                }
            }
        }
    }
    // Fallback: try connecting to external host to discover our IP
    if let Ok(addrs) = std::net::TcpStream::connect("8.8.8.8:80") {
        if let Ok(addr) = addrs.local_addr() {
            return addr.ip().to_string();
        }
    }
    "127.0.0.1".to_string()
}

fn is_ip_local(ip: &str) -> bool {
    // Try to bind a UDP socket to the given IP to see if it belongs to this machine.
    std::net::UdpSocket::bind(format!("{}:0", ip)).is_ok()
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => c,
            _ => '_',
        })
        .collect()
}
