use std::net::{TcpListener, TcpStream};

use std::env::args;
use std::fs::File;
use std::thread;
use std::{io::Read, io::Write};

use flate2::write::GzEncoder;
use flate2::Compression;

const POST: &str = "POST";
const GET: &str = "GET";

fn handle_connection(mut stream: TcpStream) {
    println!("accepted new connection");

    let mut buffer = [0; 512];
    let bytes_read = stream.read(&mut buffer).unwrap();

    let request_payload = &buffer[..bytes_read];
    let request_string = String::from_utf8(request_payload.to_vec()).unwrap();
    println!("< ----  REQUEST ---- >  \n {}", request_string);

    let split_string: Vec<&str> = request_string.split_whitespace().collect();
    let method: &str = split_string.get(0).ok_or("Failed to parse Method").unwrap();
    let path: &str = split_string
        .get(1)
        .ok_or("Failed to parse request path")
        .unwrap();

    if path == "/" {
        let _ = stream.write(b"HTTP/1.1 200 OK\r\n\r\n");
    } else if path.contains("echo") {
        let req_string = request_string.replace("\r", "");
        let split_vector_string: Vec<&str> = req_string.trim().split("\n").collect();

        let encoding: &str = split_vector_string.last().unwrap();

        let string_vector: Vec<&str> = path.split('/').collect();
        let end_path = string_vector
            .get(2)
            .ok_or("Failed to parse request path")
            .unwrap();

        println!("Contains GZIP {:?}", split_vector_string);
        let response = match encoding.contains("gzip") {
            true => {
                let mut e = GzEncoder::new(Vec::new(), Compression::default());
                let _ = e.write_all(end_path.as_bytes());
                let compressed_bytes = e
                    .finish()
                    .unwrap_or_else(|e| panic!("Error Compressing: {}", e));
                let compressed_length = compressed_bytes.len();
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length:{}\r\n\r\n",
                    compressed_length,
                )
                .into_bytes()
                .into_iter()
                .chain(compressed_bytes)
                .collect::<Vec<_>>()
            }
            _ => format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length:{}\r\n\r\n{}",
                end_path.len(),
                end_path
            )
            .into_bytes(),
        };

        let _ = stream.write_all(&response);
    } else if path.contains("user-agent") {
        let string_vector: Vec<&str> = request_string.split_whitespace().collect();
        let user_agent = string_vector.last().unwrap();
        let cleaned_user_agent: String = user_agent.replace("\r\n\r\n", "");
        let response: String = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            cleaned_user_agent.len(),
            cleaned_user_agent
        );
        let _ = stream.write(response.as_bytes());
    } else if path.contains("files") {
        let directory: String = args().last().unwrap();
        let path_vector: Vec<&str> = path.split('/').collect();
        let file_name: &&str = path_vector.last().unwrap();
        let path: String = format!("{}{}", directory, file_name);

        if method == GET {
            let mut contents = String::new();
            match File::open(path) {
                Ok(mut file) => match file.read_to_string(&mut contents) {
                    Ok(_) => {
                        let response: String = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", contents.len(), contents
                    );
                        let _ = stream.write(response.as_bytes());
                    }
                    Err(_) => {
                        let response: &str = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
                        let _ = stream.write(response.as_bytes());
                    }
                },
                Err(_) => {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        } else if method == POST {
            let request_vector: Vec<&str> = request_string.split("\n").collect();
            let payload: &str = request_vector.get(request_vector.len() - 1).unwrap();

            let mut file = File::create(path).unwrap();
            let _ = file.write_all(payload.as_bytes());

            let response = "HTTP/1.1 201 Created\r\n\r\n";
            let _ = stream.write(response.as_bytes());
        }
    } else {
        let _ = stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n");
    };
}

fn main() {
    println!("Listening on 127.0.0.1:4221");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread to handle each client connection
                thread::spawn(move || {
                    handle_connection(stream); // Call the handle_client function for each client
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
