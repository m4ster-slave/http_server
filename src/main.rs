use std::{
    io::BufReader,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
};


fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok (_stream) => {
                println!("accepted new connection");
                thread::spawn(|| {
                    handle_conn(_stream);
                });
            }
            Err (e) => {
                println!("error: {}", e);
            }
        }
    }
}


fn handle_conn(mut stream: TcpStream){
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request content: {:?}", http_request);
    let path = http_request[0].split_whitespace().nth(1).unwrap();

    let response: String;
    if path.starts_with("/echo/") {
        response = get_echo_string(path);
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
