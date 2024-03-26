use std::{fs, io::{Read, Write}, net::{TcpListener, TcpStream}, thread, time::Duration};

use simple_tcp_listener::ThreadPool;

// Handles incoming client requests.
// Reads the request from the stream and sends a response based on the request.
fn handle_client(mut stream: TcpStream) {
    // Buffer to store request
    let mut buffer = [0; 1028];

    // Populate buffer with data from stream
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let sleep = b"GET /sleep HTTP 1.1\r\n";

    let (status_line, filename) = 
        if buffer.starts_with(get) {
            ("HTTP/1.1 200 OK", "index.html")
        } else if buffer.starts_with(sleep) {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "404.html")
        };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {} \r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    // Bind open port to tcp listener
    let listener= TcpListener::bind("127.0.0.1:8080").unwrap();

    let pool = ThreadPool::new(4);

    // Listen for all incoming clients coming into the server stream
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            println!("Connection established!");
            handle_client(stream);
        });
    }
}