use std::{
    env,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
};

use http_server_starter_rust::http_request::HttpRequest;
use http_server_starter_rust::http_response;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let args: Vec<String> = env::args().collect();
    let directory = get_directory_arg(&args).unwrap_or_else(|err| {
        panic!("Error: {}", err);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let directory_ref = directory.clone();
                thread::spawn(move || {
                    handle_conn(stream, &directory_ref);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_conn(mut stream: TcpStream, directory: &str) {
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let http_request = String::from_utf8_lossy(&buffer[..]);
    let http_request: HttpRequest = separate_headers_and_body(&http_request).unwrap();
    println!("HEADERS: {:?}\n\n", http_request.headers);

    let mut response = "HTTP/1.1 400 Bad Request".to_string();
    let path = http_request.headers[0].split_whitespace().nth(1).unwrap();

    if http_request.headers[0].starts_with("GET ") {
        if path.starts_with("/echo/") {
            response = http_response::get_echo_string(path);
        } else if path.starts_with("/files/") {
            response = http_response::get_file(path, &directory);
        } else if path == "/user-agent" {
            response = http_response::get_user_agent(http_request.headers);
        } else if path == "/" {
            response = "HTTP/1.1 200 OK{http_response::CRLF}".to_string();
        } else {
            response = "HTTP/1.1 404 Not Found{http_response::CRLF}".to_string();
        }
    } else if http_request.headers[0].starts_with("POST ") {
        if path.starts_with("/files/") {
            response = http_response::post_file(&path, &directory, &http_request.body, http_request.get_body_size());

        }
    }

    stream.write_all(response.as_bytes()).unwrap();
    println!("\n");
}

fn separate_headers_and_body(input: &str) -> Option<HttpRequest> {
    if let Some(body_start) = input.find("\r\n\r\n") {
        let (headers_str, body) = input.split_at(body_start);
        let headers: Vec<&str> = headers_str.split("\r\n").collect::<Vec<&str>>();
        Some(HttpRequest::new(headers, &body[4..])) // Skip the "\r\n\r\n"
    } else {
        None
    }
}

fn get_directory_arg(args: &[String]) -> Result<String, &'static str> {
    if let Some(index) = args.iter().position(|arg| arg == "--directory") {
        if let Some(dir_arg) = args.get(index + 1) {
            return Ok(dir_arg.clone());
        }
    }
    Err("No directory argument found")
}
