use std::{
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

fn handle_conn (mut stream: TcpStream)
{
    let response = "HTTP/1.1 200 OK";
    stream.write_all(response.as_bytes()).unwrap();
}
