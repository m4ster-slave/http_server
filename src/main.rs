use std::{
    env,
    fs,
    io::BufReader,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
    path::Path,
};


fn main() {

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok (stream) => {
                println!("accepted new connection");
                thread::spawn(|| {
                    handle_conn(stream);
                });
            }
            Err (e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_conn(mut stream: TcpStream) {

    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request content: {:?}", http_request);

    let mut response = String::from("HTTP/1.1 400 Bad Request");
    let path = http_request[0].split_whitespace().nth(1).unwrap();

    if http_request[0].starts_with("GET ") {
        if path.starts_with("/echo/") {
            response = get_echo_string(path);
        } 
        else if path.starts_with("/files/") {
            let args: Vec<String> = env::args().collect();
            let directory = get_directory_arg(args).unwrap();
            println!("DIRECTORY: {}", directory);

            response = get_file(path, &directory);
        }
        else if path == "/user-agent" {
            response = get_user_agent(http_request);
        } 

        else if path == "/" {
            response = "HTTP/1.1 200 OK\r\n\r\n".to_string();
        } 
        else {
            response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
        }       
    }
    else if http_request[0].starts_with("POST ") {
        if path.starts_with("/files/") {
            let args: Vec<String> = env::args().collect();
            let directory = get_directory_arg(args).unwrap();
            println!("DIRECTORY: {}", directory);

            response = post_file(path, &directory);
        } 
    }

    stream.write_all(response.as_bytes()).unwrap();
    println!("\n\n");
}

fn get_echo_string(path: &str) -> String {
    let path = path.strip_prefix("/echo/").unwrap();
    println!("echo request: {}", path);
    const CRLF: &str = "\r\n";

    format!("HTTP/1.1 200 OK{CRLF}Content-Type: text/plain{CRLF}Content-Length: {}{CRLF}{CRLF}{path}", path.len())
}

fn get_user_agent(http_request: Vec<String>) -> String {
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

fn post_file(
    request_path: &str, 
    file_path: &str) 
-> String {
    let file_name = request_path.strip_prefix("/files/").unwrap();
    let mut full_path = String::from(file_path);
    full_path.push_str(file_name);

    println!("file posted: {}", full_path);

    let mut file = std::fs::File::create(full_path).unwrap();


    format!("HTTP/1.1 201 Created\r\n\r\n")
}
