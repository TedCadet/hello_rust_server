use hello::ThreadPool;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let thread_pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream: TcpStream = stream.unwrap();

        thread_pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader: BufReader<&mut TcpStream> = BufReader::new(&mut stream);
    // let get_home_string: &str = "GET / HTTP/1.1";
    // let http_request: Vec<_> = buf_reader
    //     .lines()
    //     .map(|result: Result<String, Error>| result.unwrap())
    //     .take_while(|line: &String| !line.is_empty())
    //     .collect();

    // println!("Request: {http_request:#?}");

    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // let (status_line, filepath) = if request_line == get_home_string {
    //     ("HTTP/1.1 200 OK", "hello.html")
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND", "404.html")
    // };

    let (status_line, filepath) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    show_page(status_line, filepath, &mut stream);
}

fn show_page(status_line: &str, filepath: &str, stream: &mut TcpStream) {
    let contents = fs::read_to_string(filepath).unwrap();

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
