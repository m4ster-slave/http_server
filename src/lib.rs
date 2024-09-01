extern crate flate2;

pub mod http_request {
    pub struct HttpRequest<'a> {
        pub headers: Vec<&'a str>,
        pub body: &'a str,
    }

    impl<'a> HttpRequest<'a> {
        pub fn new(headers: Vec<&'a str>, body: &'a str) -> Self {
            HttpRequest { headers, body }
        }

        pub fn get_body_size(&self) -> usize {
            let mut body_size = 0;
            for header in &self.headers {
                if header.starts_with("Content-Length: ") {
                    body_size = header
                        .strip_prefix("Content-Length: ")
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    break;
                }
            }

            body_size
        }
    }
}

pub mod http_response {
    use flate2::{write::GzEncoder, Compression};
    use std::{fs, io::Write, path::Path};

    const CRLF: &str = "\r\n";

    pub struct HttpResponse {
        pub headers: Vec<String>,
        pub body: Vec<u8>,
    }

    impl HttpResponse {
        pub fn create() -> HttpResponse {
            HttpResponse {
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

        pub fn return_user_agent(&mut self, http_request: &Vec<&str>) {
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

            let contents = fs::read_to_string(full_path).unwrap();

            self.headers[0] = String::from("HTTP/1.1 200 OK");
            self.headers
                .push(String::from("Content-Type: application/octet-stream"));

            self.body = Vec::from(contents);
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
            body: &str,
            max_characters: usize,
        ) {
            let file_name = request_path.strip_prefix("/files/").unwrap();
            let mut full_path = String::from(file_path);
            full_path.push_str(file_name);

            let mut file = std::fs::File::create(full_path).unwrap();

            let body_slice = &body.as_bytes()[..max_characters];
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

        pub fn compress(&mut self, request_header: &Vec<&str>) {
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
