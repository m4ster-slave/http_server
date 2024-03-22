use std::{
    env, fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                thread::spawn(|| {
                    handle_conn(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

struct HttpRequest<'a> {
    headers: Vec<&'a str>,
    body: &'a str,
}

impl<'a> HttpRequest<'a> {
    fn new(headers: Vec<&'a str>, body: &'a str) -> Self {
        HttpRequest { headers, body }
    }
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

fn handle_conn(mut stream: TcpStream) {
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let http_request = String::from_utf8_lossy(&buffer[..]);
    let http_request: HttpRequest = separate_headers_and_body(&http_request).unwrap();
    println!("HEADERS: {:?}\n\n", http_request.headers);

    let mut response = String::from("HTTP/1.1 400 Bad Request");
    let path = http_request.headers[0].split_whitespace().nth(1).unwrap();

    if http_request.headers[0].starts_with("GET ") {
        if path.starts_with("/echo/") {
            response = get_echo_string(path);
        } else if path.starts_with("/files/") {
            let args: Vec<String> = env::args().collect();
            let directory = get_directory_arg(args).unwrap();
            println!("DIRECTORY: {}", directory);

            response = get_file(path, &directory);
        } else if path == "/user-agent" {
            response = get_user_agent(http_request.headers);
        } else if path == "/" {
            response = "HTTP/1.1 200 OK\r\n\r\n".to_string();
        } else {
            response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
        }
    } else if http_request.headers[0].starts_with("POST ") {
        if path.starts_with("/files/") {
            let args: Vec<String> = env::args().collect();
            let directory = get_directory_arg(args).unwrap();
            response = post_file(&path, &directory, &http_request.body, get_body_size(http_request.headers));
        }
    }

    stream.write_all(response.as_bytes()).unwrap();
    println!("\n\n");
}

fn get_echo_string(path: &str) -> String {
    let path = path.strip_prefix("/echo/").unwrap();
    println!("echo request: {}", path);
    const CRLF: &str = "\r\n";

    format!(
        "HTTP/1.1 200 OK{CRLF}Content-Type: text/plain{CRLF}Content-Length: {}{CRLF}{CRLF}{path}",
        path.len()
    )
}

fn get_user_agent(http_request: Vec<&str>) -> String {
    let mut user_agent = String::from("no user agent in request header");
    for s in http_request {
        if s.starts_with("User-Agent: ") {
            user_agent = s.strip_prefix("User-Agent: ").unwrap().to_string();
            break;
        }
    }

    println!("user_agent: {}", user_agent);
    const CRLF: &str = "\r\n";
    format!("HTTP/1.1 200 OK{CRLF}Content-Type: text/plain{CRLF}Content-Length: {}{CRLF}{CRLF}{user_agent}", user_agent.len())
}

fn get_directory_arg(args: Vec<String>) -> Option<String> {
    if let Some(index) = args.iter().position(|arg| arg == "--directory") {
        if let Some(dir_arg) = args.get(index + 1) {
            return Some(dir_arg.clone());
        }
    }
    panic!("no directory set\n");
}

fn get_body_size(http_request: Vec<&str>) -> usize {
    let mut body_size = 0;
        for s in http_request {
        if s.starts_with("Content-Length: ") {
            body_size = s.strip_prefix("Content-Length: ").unwrap().parse::<usize>().unwrap();
            break;
        }
    }

    body_size
}

fn get_file(request_path: &str, file_path: &str) -> String {
    let file_name = request_path.strip_prefix("/files/").unwrap();
    let mut full_path = String::from(file_path);
    full_path.push_str(file_name);

    println!("file requested: {}", full_path);
    if !Path::new(&full_path).exists() {
        return "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
    }

    let contents = fs::read_to_string(full_path).unwrap();

    const CRLF: &str = "\r\n";
    format!("HTTP/1.1 200 OK{CRLF}Content-Type: application/octet-stream{CRLF}Content-Length: {}{CRLF}{CRLF}{contents}", contents.len())
}

fn post_file(request_path: &str, file_path: &str, body: &str, max_characters: usize) -> String {
    let file_name = request_path.strip_prefix("/files/").unwrap();
    let mut full_path = String::from(file_path);
    full_path.push_str(file_name);
    println!("FILE: {}", full_path);
    println!("BODY: {}", body);

    println!("file posted: {}", full_path);
    let mut file = std::fs::File::create(full_path).unwrap();

    let body_slice = &body.as_bytes()[..max_characters];
    file.write_all(body_slice).unwrap();

    format!("HTTP/1.1 201 Created\r\n\r\n")
}
