use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

use flate2::write::GzEncoder;
use flate2::Compression;

#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
    Unsupported,
}

struct Request {
    method: HttpMethod,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn parse_request(buffer: &[u8]) -> Result<Request, Box<dyn std::error::Error>> {
    let request_str = String::from_utf8_lossy(buffer);
    let lines: Vec<&str> = request_str.lines().collect();

    if lines.is_empty() {
        return Err("Empty request".into());
    }

    let first_line: Vec<&str> = lines[0].split_whitespace().collect();
    if first_line.len() < 2 {
        return Err("Invalid request line".into());
    }

    let method = match first_line[0] {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        _ => HttpMethod::Unsupported,
    };

    let path = first_line[1].to_string();

    let mut headers = Vec::new();
    let mut body_start = 0;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
        if let Some((key, value)) = line.split_once(": ") {
            headers.push((key.to_lowercase(), value.to_string()));
        }
    }

    let body = if body_start < lines.len() {
        lines[body_start..].join("\n").into_bytes()
    } else {
        Vec::new()
    };

    Ok(Request {
        method,
        path,
        headers,
        body,
    })
}

fn handle_connection(mut stream: TcpStream) {
    println!("Accepted new connection");

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();

    match parse_request(&buffer[..bytes_read]) {
        Ok(request) => handle_request(stream, request),
        Err(e) => {
            eprintln!("Failed to parse request: {}", e);
            let _ = stream.write(b"HTTP/1.1 400 Bad Request\r\n\r\n");
        }
    }
}

fn handle_request(mut stream: TcpStream, request: Request) {
    match request.path.as_str() {
        "/" => send_response(&mut stream, "HTTP/1.1 200 OK\r\n\r\n"),
        path if path.starts_with("/echo/") => handle_echo(&mut stream, &request),
        "/user-agent" => handle_user_agent(&mut stream, &request),
        path if path.starts_with("/files/") => handle_files(&mut stream, &request),
        _ => send_response(&mut stream, "HTTP/1.1 404 Not Found\r\n\r\n"),
    }
}

fn send_response(stream: &mut TcpStream, response: &str) {
    let _ = stream.write(response.as_bytes());
}

fn handle_echo(stream: &mut TcpStream, request: &Request) {
    let content = request.path.split('/').nth(2).unwrap_or("");
    let is_gzip = request
        .headers
        .iter()
        .any(|(k, v)| k == "accept-encoding" && v.contains("gzip"));

    let response = if is_gzip {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(content.as_bytes()).unwrap();
        let compressed = e.finish().unwrap();
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n",
            compressed.len()
        ).into_bytes().into_iter().chain(compressed).collect()
    } else {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        )
        .into_bytes()
    };

    let _ = stream.write_all(&response);
}

fn handle_user_agent(stream: &mut TcpStream, request: &Request) {
    if let Some((_, user_agent)) = request.headers.iter().find(|(k, _)| k == "user-agent") {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            user_agent.len(),
            user_agent
        );
        let _ = stream.write(response.as_bytes());
    } else {
        send_response(stream, "HTTP/1.1 400 Bad Request\r\n\r\n");
    }
}

fn handle_files(stream: &mut TcpStream, request: &Request) {
    let directory = env::args().last().unwrap_or_else(|| ".".to_string());
    let file_name = request.path.split('/').last().unwrap_or("");
    let path = Path::new(&directory).join(file_name);

    match request.method {
        HttpMethod::Get => match fs::read(&path) {
            Ok(contents) => {
                let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                        contents.len()
                    );
                let _ = stream.write(response.as_bytes());
                let _ = stream.write(&contents);
            }
            Err(_) => send_response(stream, "HTTP/1.1 404 Not Found\r\n\r\n"),
        },
        HttpMethod::Post => {
            if let Err(_) = fs::write(&path, &request.body) {
                send_response(stream, "HTTP/1.1 500 Internal Server Error\r\n\r\n");
            } else {
                send_response(stream, "HTTP/1.1 201 Created\r\n\r\n");
            }
        }
        _ => send_response(stream, "HTTP/1.1 405 Method Not Allowed\r\n\r\n"),
    }
}

fn main() {
    let address = "127.0.0.1:4221";
    println!("Listening on {}", address);
    let listener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
