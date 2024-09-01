use std::{
    env,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
};

use http_server::{http_request::HttpRequest, http_response::HttpResponse};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Listening on 127.0.0.1:8080");

    let args: Vec<String> = env::args().collect();
    let directory = get_directory_arg(&args).unwrap_or_else(|err| {
        panic!("Error: {}", err);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                print!("[+] Accepted new connection");
                print!("({})\t", stream.peer_addr().unwrap());
                let directory_ref = directory.clone();
                thread::spawn(move || {
                    handle_conn(stream, &directory_ref);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn handle_conn(mut stream: TcpStream, directory: &str) {
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let http_request = String::from_utf8_lossy(&buffer[..]);
    let http_request: HttpRequest = separate_headers_and_body(&http_request).unwrap();
    print!("{}\n", http_request.headers[0]);

    let mut response = HttpResponse::create();
    let path = http_request.headers[0].split_whitespace().nth(1).unwrap();

    if http_request.headers[0].starts_with("GET ") {
        if path.starts_with("/echo/") {
            response.return_echo_string(path);
        } else if path.starts_with("/files/") {
            response.get_file(path, &directory);
        } else if path == "/user-agent" {
            response.return_user_agent(&http_request.headers);
        } else if path == "/" {
            response.return_status_code(200);
        } else {
            response.return_status_code(404);
        }
    } else if http_request.headers[0].starts_with("POST ") {
        if path.starts_with("/files/") {
            response.post_file(
                &path,
                &directory,
                &http_request.body,
                http_request.get_body_size(),
            );
        }
    }

    // compress iff gzip compression is availaible by client
    // and set "Content-Encoding" header
    response.compress(&http_request.headers);

    // set the content length
    response.headers
        .push(format!("Content-Length: {}", response.body.len()));

    // write response into tcp stream 
    stream
        .write_all(&response.write_response())
        .unwrap();
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

// fn compress_response(response: String, request_header: &Vec<&str>) -> String {}
