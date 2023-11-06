use std::io::{BufRead, Read, Result, Write};
use std::str::FromStr;
use std::{io::BufReader, net::TcpStream, time::Duration};

use crate::{http_request::HttpRequest, method_verb::MethodVerb};

const BAD_REQUEST_RESPONSE: &str = r"HTTP/1.1 400 Bad Request

Bad Request";

const OK_RESPONSE: &str = r"HTTP/1.1 200 OK

";

const NOT_FOUND_RESPONSE: &str = r"HTTP/1.1 404 Not Found

";

const CONTENT_LENGTH_HEADER: &str = "content-length:";
const STREAM_READ_TIMEOUT: u64 = 5;

pub fn return_bad_request(stream: std::net::TcpStream) -> () {
    return_response(stream, BAD_REQUEST_RESPONSE.to_string());
}

pub fn return_not_found(stream: std::net::TcpStream) -> () {
    return_response(stream, NOT_FOUND_RESPONSE.to_string());
}

pub fn return_ok_response(stream: std::net::TcpStream, body: &str) -> () {
    return_response(stream, format!("{OK_RESPONSE}{body}"));
}

pub fn parse_request(stream: std::net::TcpStream) -> Option<HttpRequest> {
    stream
        .set_read_timeout(Some(Duration::from_secs(STREAM_READ_TIMEOUT)))
        .unwrap();

    let mut reader = BufReader::new(&stream);
    let mut start_line = String::new();

    if reader.read_line(&mut start_line).is_err() {
        return None;
    }

    let mut start_line_iter = start_line.split_whitespace();
    let verb = start_line_iter.next().unwrap_or("");
    let uri = start_line_iter.next().unwrap_or("");

    let verb = MethodVerb::from_str(verb);

    if verb.is_err() || uri.is_empty() {
        return None;
    }

    let verb = verb.unwrap();

    if !could_have_body(&verb) {
        return Some(HttpRequest {
            method: verb,
            uri: uri.to_string(),
            body: None,
        });
    }

    let headers = read_headers(&mut reader);

    if headers.is_err() {
        return None;
    }

    let headers = headers.unwrap();

    let body_length = get_content_length_header(&headers).unwrap_or(0);

    if body_length == 0 {
        return Some(HttpRequest {
            method: verb,
            uri: uri.to_string(),
            body: None,
        });
    }

    let body = read_body(&mut reader, body_length);

    if body.is_err() {
        return None;
    }
    let body = body.unwrap();

    return Some(HttpRequest {
        method: verb,
        uri: uri.to_string(),
        body: Some(body),
    });
}

fn read_body(reader: &mut BufReader<&TcpStream>, body_length: usize) -> Result<String> {
    let mut buffer = vec![0; body_length];
    let mut bytes_num = 0;

    loop {
        bytes_num += reader.read(&mut buffer)?;

        if bytes_num == body_length {
            break;
        }
    }

    Ok(String::from_utf8(buffer).unwrap())
}

fn get_content_length_header(headers: &String) -> Option<usize> {
    let start_idx = headers.to_lowercase().find(CONTENT_LENGTH_HEADER);

    if start_idx.is_none() {
        return None;
    }

    let start_idx = start_idx.unwrap();
    let end_idx = headers.get(start_idx..).unwrap().find("\n").unwrap() + start_idx;

    let content_length_header = headers.get(start_idx..end_idx).unwrap();
    let content_length = content_length_header.split(':').into_iter().last().unwrap();

    match content_length.trim().parse::<usize>() {
        Ok(length) => Some(length),
        Err(_) => None,
    }
}

fn read_headers(reader: &mut BufReader<&TcpStream>) -> Result<String> {
    let mut buffer = String::new();
    let mut current_string = String::new();

    loop {
        current_string.clear();
        reader.read_line(&mut current_string)?;

        if current_string == "\r\n" {
            break;
        }

        buffer = format!("{buffer}{current_string}");
    }

    Ok(buffer)
}

fn could_have_body(method: &MethodVerb) -> bool {
    match method {
        MethodVerb::Get | MethodVerb::Delete => false,
        MethodVerb::Post | MethodVerb::Put => true,
    }
}

fn return_response(mut stream: std::net::TcpStream, response: String) -> () {
    stream
        .write_all(response.as_bytes())
        .unwrap_or_else(|e| println!("{e}"));
}
