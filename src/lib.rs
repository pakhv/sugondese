use std::{
    io::{BufRead, BufReader, Lines, Result, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
    time::Duration,
};

use crate::method_verb::MethodVerb;

mod method_verb;

const RESOURCE_NOT_FOUND_RESPONSE: &str = r"HTTP/1.1 404 Not Found

Resource not found";

pub struct WebApi<'a> {
    addr: &'a str,
}

impl<'a> WebApi<'a> {
    pub fn new(addr: &'a str) -> WebApi {
        WebApi { addr }
    }

    pub fn run(&mut self) -> Result<()> {
        let tcp_listener = TcpListener::bind(self.addr)?;

        for stream_result in tcp_listener.incoming() {
            if let Ok(stream) = stream_result {
                handle_connection(stream);
            }
        }

        Ok(())
    }

    pub fn get<F>(_handler: F) -> ()
    where
        F: FnOnce(),
    {
    }
}

fn handle_connection(stream: std::net::TcpStream) -> () {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    let mut reader = BufReader::new(&stream);
    let mut buffer = String::new();

    if reader.read_line(&mut buffer).is_err() {
        return_not_found(stream);
        return;
    }

    let verb = buffer.split_whitespace().next().unwrap_or("");
    let uri = buffer.split_whitespace().next().unwrap_or("");

    let verb = MethodVerb::from_str(verb);

    if verb.is_err() || uri.is_empty() {
        return_not_found(stream);
        return;
    }

    let mut request_lines = reader.lines();
    let headers = read_until_empty_string(&mut request_lines);
    let body = read_until_empty_string(&mut request_lines);

    println!("{buffer}");
    println!("{headers}");
    println!("{body}");
}

fn read_until_empty_string(iter: &mut Lines<BufReader<&TcpStream>>) -> String {
    let mut buffer = String::new();

    loop {
        let line = iter.next().unwrap();

        match line {
            Ok(line) => {
                if line == "" {
                    break;
                }

                buffer = format!("{buffer}\n{line}");
            }
            Err(_) => break,
        }
    }

    buffer
}

fn return_not_found(mut stream: std::net::TcpStream) -> () {
    stream
        .write_all(RESOURCE_NOT_FOUND_RESPONSE.as_bytes())
        .unwrap_or_else(|e| println!("{e}"));
}
