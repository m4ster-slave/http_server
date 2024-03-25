
pub mod http_request {
    pub struct HttpRequest<'a> {
        pub headers: Vec<&'a str>,
        pub body: &'a str,
    }

    impl<'a> HttpRequest<'a> {
        pub fn new(headers: Vec<&'a str>, body: &'a str) -> Self {
            HttpRequest { headers, body }
        }

        pub fn get_body_size(self) -> usize {
            let mut body_size = 0;
                for header in self.headers {
                if header.starts_with("Content-Length: ") {
                    body_size = header.strip_prefix("Content-Length: ").unwrap().parse::<usize>().unwrap();
                    break;
                }
            }

            body_size
        }
    }

}

pub mod http_response {
    use std::{
        fs,
        path::Path,
        io::prelude::*,
    };

    const CRLF: &str = "\r\n";

    pub fn get_echo_string(path: &str) -> String {
        let path = path.strip_prefix("/echo/").unwrap();
        println!("echo request: {}", path);

        format!(
            "HTTP/1.1 200 OK{CRLF}Content-Type: text/plain{CRLF}Content-Length: {}{CRLF}{CRLF}{path}",
            path.len()
        )
    }

    pub fn get_user_agent(http_request: Vec<&str>) -> String {
        let mut user_agent = "no user agent in request header".to_string();
        for s in http_request {
            if s.starts_with("User-Agent: ") {
                user_agent = s.strip_prefix("User-Agent: ").unwrap().to_string();
                break;
            }
        }

        println!("user_agent: {}", user_agent);
        format!("HTTP/1.1 200 OK{CRLF}Content-Type: text/plain{CRLF}Content-Length: {}{CRLF}{CRLF}{user_agent}", user_agent.len())
    }

    pub fn get_file(request_path: &str, file_path: &str) -> String {
        let file_name = request_path.strip_prefix("/files/").unwrap();
        let mut full_path = String::from(file_path);
        full_path.push_str(file_name);

        println!("file requested: {}", full_path);
        if !Path::new(&full_path).exists() {
            return "HTTP/1.1 404 Not Found{CRLF}".to_string();
        }

        let contents = fs::read_to_string(full_path).unwrap();

        format!("HTTP/1.1 200 OK{CRLF}Content-Type: application/octet-stream{CRLF}Content-Length: {}{CRLF}{CRLF}{contents}", contents.len())
    }
    

    pub fn post_file(request_path: &str, file_path: &str, body: &str, max_characters: usize) -> String {
        let file_name = request_path.strip_prefix("/files/").unwrap();
        let mut full_path = String::from(file_path);
        full_path.push_str(file_name);
        println!("FILE: {}", full_path);
        println!("BODY: {}", body);

        println!("file posted: {}", full_path);
        let mut file = std::fs::File::create(full_path).unwrap();

        let body_slice = &body.as_bytes()[..max_characters];
        file.write_all(body_slice).unwrap();

        format!("HTTP/1.1 201 Created{CRLF}")
    }
}
