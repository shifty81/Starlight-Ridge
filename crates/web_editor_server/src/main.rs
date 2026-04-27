use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

const DEFAULT_PORT: u16 = 8787;
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

fn main() {
    let root = find_project_root().unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let host = env::var("STARLIGHT_WEB_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("STARLIGHT_WEB_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);
    let write_enabled = env::var("STARLIGHT_WEB_ALLOW_WRITE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);

    let bind_addr = format!("{host}:{port}");
    let listener = match TcpListener::bind(&bind_addr) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Failed to bind web editor server to {bind_addr}: {error}");
            eprintln!("Try a different port: STARLIGHT_WEB_PORT=8788 cargo run -p web_editor_server");
            std::process::exit(1);
        }
    };

    println!("========================================");
    println!(" Starlight Ridge Web Editor LAN Server");
    println!(" Root: {}", root.display());
    println!(" Bind: {bind_addr}");
    println!(" Write mode: {}", if write_enabled { "ENABLED" } else { "read-only" });
    println!("----------------------------------------");
    println!(" Open on this PC:     http://127.0.0.1:{port}/");
    println!(" View picker:         http://127.0.0.1:{port}/?launcher=1");
    if let Some(ip) = local_lan_ip() {
        println!(" Open on phone/LAN:  http://{ip}:{port}/");
        println!(" Force tablet mode:  http://{ip}:{port}/?mode=tablet");
        println!(" Force mobile mode:  http://{ip}:{port}/?mode=mobile");
        println!(" Open view picker:   http://{ip}:{port}/?launcher=1");
    } else {
        println!(" Open on phone/LAN:  http://<this-PC-LAN-IP>:{port}/");
        println!(" Force tablet mode:  http://<this-PC-LAN-IP>:{port}/?mode=tablet");
        println!(" Force mobile mode:  http://<this-PC-LAN-IP>:{port}/?mode=mobile");
        println!(" Open view picker:   http://<this-PC-LAN-IP>:{port}/?launcher=1");
    }
    println!("----------------------------------------");
    println!(" If your tablet cannot connect, allow this port through Windows Firewall.");
    println!(" To enable saving from browser: set STARLIGHT_WEB_ALLOW_WRITE=1 before launch.");
    println!("========================================");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                if let Err(error) = handle_connection(&mut stream, &root, write_enabled) {
                    let body = format!("Internal server error: {error}\n");
                    let _ = write_response(&mut stream, 500, "text/plain; charset=utf-8", body.as_bytes());
                }
            }
            Err(error) => eprintln!("Connection failed: {error}"),
        }
    }
}

fn find_project_root() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("content").is_dir() && dir.join("assets").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn local_lan_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}

fn handle_connection(stream: &mut TcpStream, root: &Path, write_enabled: bool) -> std::io::Result<()> {
    let mut buffer = vec![0_u8; 8192];
    let mut bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let mut request = buffer[..bytes_read].to_vec();
    while !request.windows(4).any(|window| window == b"\r\n\r\n") && request.len() < 64 * 1024 {
        bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..bytes_read]);
    }

    let header_end = match request.windows(4).position(|window| window == b"\r\n\r\n") {
        Some(index) => index + 4,
        None => return write_response(stream, 400, "text/plain; charset=utf-8", b"Bad request\n"),
    };

    let headers = String::from_utf8_lossy(&request[..header_end]);
    let mut lines = headers.lines();
    let request_line = match lines.next() {
        Some(line) => line,
        None => return write_response(stream, 400, "text/plain; charset=utf-8", b"Bad request\n"),
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let raw_path = parts.next().unwrap_or("/");
    let path = raw_path.split('?').next().unwrap_or("/");

    let request_host = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("host") {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    if content_length > MAX_BODY_BYTES {
        return write_response(stream, 413, "text/plain; charset=utf-8", b"Request body too large\n");
    }

    let mut body = request[header_end..].to_vec();
    while body.len() < content_length {
        bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..bytes_read]);
    }
    body.truncate(content_length);

    if method == "OPTIONS" {
        return write_options(stream);
    }

    match (method, path) {
        ("GET", "/") => serve_file(stream, &root.join("tools/web_editor/index.html")),
        ("GET", "/api/health") => {
            let json = format!(
                "{{\"ok\":true,\"write_enabled\":{},\"root\":\"{}\"}}",
                write_enabled,
                json_escape(&root.display().to_string())
            );
            write_response(stream, 200, "application/json; charset=utf-8", json.as_bytes())
        }
        ("GET", "/api/server_info") => serve_server_info(stream, root, write_enabled, &request_host),
        ("GET", "/api/manifest") => serve_manifest(stream, root, write_enabled, &request_host),
        _ if method == "GET" && path.starts_with("/api/map/") => serve_map_bundle(stream, root, &path[9..]),
        _ if method == "POST" && path.starts_with("/api/save/map/") => save_map_layers(stream, root, &path[14..], &body, write_enabled),
        _ if method == "GET" && path.starts_with("/web/") => {
            let rel = &path[5..];
            serve_safe_file(stream, &root.join("tools/web_editor"), rel)
        }
        _ if method == "GET" && path.starts_with("/assets/") => serve_safe_file(stream, root, &path[1..]),
        _ if method == "GET" && path.starts_with("/content/") => serve_safe_file(stream, root, &path[1..]),
        _ => write_response(stream, 404, "text/plain; charset=utf-8", b"Not found\n"),
    }
}

fn serve_manifest(stream: &mut TcpStream, root: &Path, write_enabled: bool, request_host: &str) -> std::io::Result<()> {
    let mut maps = Vec::new();
    let maps_dir = root.join("content/maps");
    if let Ok(entries) = fs::read_dir(maps_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("map.ron").is_file() {
                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                    maps.push(name.to_string());
                }
            }
        }
    }
    maps.sort();

    let mut tilesets = Vec::new();
    let tiles_dir = root.join("content/tiles");
    if let Ok(entries) = fs::read_dir(tiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("ron") {
                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                    if !is_editor_sidecar_tileset(name) {
                        tilesets.push(name.to_string());
                    }
                }
            }
        }
    }
    tilesets.sort();

    let maps_json = maps.iter().map(|value| format!("\"{}\"", json_escape(value))).collect::<Vec<_>>().join(",");
    let tilesets_json = tilesets.iter().map(|value| format!("\"{}\"", json_escape(value))).collect::<Vec<_>>().join(",");
    let json = format!(
        "{{\"project\":\"Starlight Ridge\",\"write_enabled\":{},\"mode\":\"lan_companion\",\"pc_url\":\"http://{}?mode=pc\",\"tablet_url\":\"http://{}?mode=tablet\",\"mobile_url\":\"http://{}?mode=mobile\",\"launcher_url\":\"http://{}?launcher=1\",\"maps\":[{}],\"tilesets\":[{}]}}",
        write_enabled,
        json_escape(request_host),
        json_escape(request_host),
        json_escape(request_host),
        json_escape(request_host),
        maps_json,
        tilesets_json
    );
    write_response(stream, 200, "application/json; charset=utf-8", json.as_bytes())
}

fn serve_server_info(
    stream: &mut TcpStream,
    root: &Path,
    write_enabled: bool,
    request_host: &str,
) -> std::io::Result<()> {
    let json = format!(
        "{{\"project\":\"Starlight Ridge\",\"mode\":\"lan_companion\",\"write_enabled\":{},\"root\":\"{}\",\"pc_url\":\"http://{}?mode=pc\",\"tablet_url\":\"http://{}?mode=tablet\",\"mobile_url\":\"http://{}?mode=mobile\",\"launcher_url\":\"http://{}?launcher=1\"}}",
        write_enabled,
        json_escape(&root.display().to_string()),
        json_escape(request_host),
        json_escape(request_host),
        json_escape(request_host),
        json_escape(request_host)
    );
    write_response(stream, 200, "application/json; charset=utf-8", json.as_bytes())
}

fn serve_map_bundle(stream: &mut TcpStream, root: &Path, map_id: &str) -> std::io::Result<()> {
    if !is_safe_segment(map_id) {
        return write_response(stream, 400, "text/plain; charset=utf-8", b"Bad map id\n");
    }
    let map_dir = root.join("content/maps").join(map_id);
    if !map_dir.is_dir() {
        return write_response(stream, 404, "text/plain; charset=utf-8", b"Map not found\n");
    }

    let map_ron = read_to_string_or_empty(&map_dir.join("map.ron"));
    let layers_ron = read_to_string_or_empty(&map_dir.join("layers.ron"));
    let props_ron = read_to_string_or_empty(&map_dir.join("props.ron"));
    let spawns_ron = read_to_string_or_empty(&map_dir.join("spawns.ron"));
    let triggers_ron = read_to_string_or_empty(&map_dir.join("triggers.ron"));
    let tileset_file = tileset_file_for(&map_ron);
    let tileset_ron = read_to_string_or_empty(&root.join("content/tiles").join(&tileset_file));

    let json = format!(
        "{{\"id\":\"{}\",\"map_ron\":\"{}\",\"layers_ron\":\"{}\",\"props_ron\":\"{}\",\"spawns_ron\":\"{}\",\"triggers_ron\":\"{}\",\"tileset_file\":\"{}\",\"tileset_ron\":\"{}\"}}",
        json_escape(map_id),
        json_escape(&map_ron),
        json_escape(&layers_ron),
        json_escape(&props_ron),
        json_escape(&spawns_ron),
        json_escape(&triggers_ron),
        json_escape(&tileset_file),
        json_escape(&tileset_ron),
    );
    write_response(stream, 200, "application/json; charset=utf-8", json.as_bytes())
}

fn save_map_layers(
    stream: &mut TcpStream,
    root: &Path,
    tail: &str,
    body: &[u8],
    write_enabled: bool,
) -> std::io::Result<()> {
    if !write_enabled {
        return write_response(
            stream,
            403,
            "application/json; charset=utf-8",
            b"{\"ok\":false,\"error\":\"Write mode disabled. Relaunch with STARLIGHT_WEB_ALLOW_WRITE=1.\"}",
        );
    }

    let Some((map_id, file_name)) = tail.split_once('/') else {
        return write_response(stream, 400, "text/plain; charset=utf-8", b"Bad save path\n");
    };
    if !is_safe_segment(map_id) || file_name != "layers.ron" {
        return write_response(stream, 400, "text/plain; charset=utf-8", b"Only map layers.ron can be saved from this early web editor\n");
    }

    let text = match String::from_utf8(body.to_vec()) {
        Ok(text) => text,
        Err(_) => return write_response(stream, 400, "text/plain; charset=utf-8", b"Body must be UTF-8 text\n"),
    };
    if !text.contains("layers:") || !text.contains("rows:") {
        return write_response(stream, 400, "text/plain; charset=utf-8", b"layers.ron validation failed: missing layers/rows\n");
    }

    let map_dir = root.join("content/maps").join(map_id);
    let target = map_dir.join("layers.ron");
    if !target.is_file() {
        return write_response(stream, 404, "text/plain; charset=utf-8", b"Target layers.ron not found\n");
    }

    let backup = map_dir.join("layers.ron.web_backup");
    let _ = fs::copy(&target, backup);
    fs::write(&target, text)?;
    write_response(stream, 200, "application/json; charset=utf-8", b"{\"ok\":true}")
}

fn read_to_string_or_empty(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn tileset_file_for(map_ron: &str) -> String {
    let tileset_id = extract_quoted_after(map_ron, "tileset:").unwrap_or_else(|| "base_tiles".to_string());
    match tileset_id.as_str() {
        "base_tiles" => "base_tileset.ron".to_string(),
        "base_tiles_spring" => "base_tiles_spring.ron".to_string(),
        "base_tiles_summer" => "base_tiles_summer.ron".to_string(),
        "base_tiles_autumn" => "base_tiles_autumn.ron".to_string(),
        "base_tiles_winter" => "base_tiles_winter.ron".to_string(),
        other => format!("{other}.ron"),
    }
}

fn extract_quoted_after(text: &str, key: &str) -> Option<String> {
    let start = text.find(key)? + key.len();
    let rest = &text[start..];
    let first = rest.find('"')? + 1;
    let tail = &rest[first..];
    let end = tail.find('"')?;
    Some(tail[..end].to_string())
}

fn serve_safe_file(stream: &mut TcpStream, base: &Path, relative: &str) -> std::io::Result<()> {
    let mut target = base.to_path_buf();
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => target.push(part),
            _ => return write_response(stream, 400, "text/plain; charset=utf-8", b"Bad path\n"),
        }
    }
    serve_file(stream, &target)
}

fn serve_file(stream: &mut TcpStream, path: &Path) -> std::io::Result<()> {
    if !path.is_file() {
        return write_response(stream, 404, "text/plain; charset=utf-8", b"File not found\n");
    }
    let content = fs::read(path)?;
    let content_type = content_type_for(path);
    write_response(stream, 200, content_type, &content)
}

fn write_options(stream: &mut TcpStream) -> std::io::Result<()> {
    let response = "HTTP/1.1 204 No Content\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: Content-Type\r\n\
Content-Length: 0\r\n\
Connection: close\r\n\r\n";
    stream.write_all(response.as_bytes())
}

fn write_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\n\
Content-Type: {content_type}\r\n\
Content-Length: {}\r\n\
Access-Control-Allow-Origin: *\r\n\
Cache-Control: no-store\r\n\
Connection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "ron" | "txt" | "md" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

fn is_editor_sidecar_tileset(file_name: &str) -> bool {
    file_name.ends_with("_roles.ron")
        || file_name.ends_with("_rules.ron")
        || file_name.contains("sidecar")
}

fn is_safe_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}
