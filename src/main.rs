use std::{
    io::BufReader,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

fn main() 
{
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() 
    {
            match stream 
            {
                Ok (_stream) => 
                {
                    println!("accepted new connection");
                    handle_conn(_stream);
                }
                Err (e) => 
                {
                    println!("error: {}", e);
                }
            }


     }
}

fn handle_conn(mut stream: TcpStream)
{
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request content: {:?}", http_request);

    let response;
    if http_request.into_iter().nth(0) == Some(String::from("GET / HTTP/1.1"))
    {
         response = "HTTP/1.1 200 OK\r\n\r\n";
    }
    else 
    {
        response = "HTTP/1.1 404\r\n\r\n";
    }

    stream.write_all(response.as_bytes()).unwrap();
}
