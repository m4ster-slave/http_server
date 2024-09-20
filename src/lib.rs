extern crate flate2;

pub mod http_msg {
    pub struct HttpMsg {
        pub headers: Vec<String>,
        pub body: Vec<u8>,
    }
}

pub mod http_request {
    use crate::http_msg::HttpMsg;

    impl HttpMsg {
        pub fn from(input: &str) -> HttpMsg {
            // spereate header from the body
            if let Some(body_start) = input.find("\r\n\r\n") {
                let (headers_str, body_str) = input.split_at(body_start);

                HttpMsg {
                    headers: headers_str
                        .split("\r\n")
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),

                    body: body_str.as_bytes().to_vec(),
                }
            } else {
                panic!("Not a valid request");
            }
        }
    }
}

pub mod http_response {
    use crate::http_msg::HttpMsg;
    use flate2::{write::GzEncoder, Compression};
    use std::{fs, io::Write, path::Path};

    const CRLF: &str = "\r\n";

    impl HttpMsg {
        pub fn create() -> HttpMsg {
            HttpMsg {
                headers: vec![String::from("HTTP/1.1 400 Bad Request")],
                body: Vec::new(),
            }
        }

        pub fn return_echo_string(&mut self, path: &str) {
            let path = path.strip_prefix("/echo/").unwrap();

            self.headers[0] = String::from("HTTP/1.1 200 OK");
            self.headers.push(String::from("Content-Type: text/plain"));

            self.body = Vec::from(path);
        }

        pub fn return_user_agent(&mut self, http_request: &Vec<String>) {
            let mut user_agent = "no user agent in request header".to_string();

            for s in http_request {
                if s.starts_with("User-Agent: ") {
                    user_agent = s.strip_prefix("User-Agent: ").unwrap().to_string();
                    break;
                }
            }

            self.headers[0] = String::from("HTTP/1.1 200 OK");
            self.headers.push(String::from("Content-Type: text/plain"));

            self.body = Vec::from(user_agent);
        }

        pub fn get_file(&mut self, request_path: &str, file_path: &str) {
            let file_name = request_path.strip_prefix("/files/").unwrap();
            let mut full_path = String::from(file_path);
            full_path.push_str(file_name);

            if !Path::new(&full_path).exists() {
                self.return_status_code(404);
                return;
            }

            self.body = fs::read(full_path).unwrap();

            self.headers[0] = String::from("HTTP/1.1 200 OK");
            self.headers
                .push(String::from("Content-Type: application/octet-stream"));
        }

        pub fn return_status_code(&mut self, status_code: u32) {
            match status_code {
                200 => self.headers[0] = String::from("HTTP/1.1 200 OK"),
                404 => self.headers[0] = String::from("HTTP/1.1 404 Not Found"),
                201 => self.headers[0] = String::from("HTTP/1.1 201 Created"),
                405 => self.headers[0] = String::from("HTTP/1.1 405 Method Not Allowed"),
                _ => panic!("Status code unimplemented"),
            }
        }

        pub fn post_file(
            &mut self,
            request_path: &str,
            file_path: &str,
            body: Vec<u8>,
            max_characters: usize,
        ) {
            let file_name = request_path.strip_prefix("/files/").unwrap();
            let mut full_path = String::from(file_path);
            full_path.push_str(file_name);

            let mut file = std::fs::File::create(full_path).unwrap();

            let body_slice = &body[..max_characters];
            file.write_all(body_slice).unwrap();

            self.return_status_code(201);
        }

        pub fn write_response(&self) -> Vec<u8> {
            let mut response = Vec::new();

            // Convert headers to a single string with CRLF
            for s in &self.headers {
                response.extend_from_slice(s.as_bytes());
                response.extend_from_slice(CRLF.as_bytes());
            }
            // Add an extra CRLF to separate headers from the body
            response.extend_from_slice(CRLF.as_bytes());

            // Append the body to the response
            response.extend_from_slice(&self.body);

            response
        }

        pub fn compress(&mut self, request_header: &Vec<String>) {
            if !self.body.is_empty() {
                for s in request_header {
                    if s.starts_with("Accept-Encoding: ") && s.contains("gzip") {
                        self.headers.push(String::from("Content-Encoding: gzip"));
                        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                        encoder.write_all(&self.body).unwrap();
                        let compressed_body = encoder.finish().unwrap();

                        self.body = compressed_body.to_vec();
                        break;
                    }
                }
            }
        }
    }
}
