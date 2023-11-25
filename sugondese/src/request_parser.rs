use std::collections::HashMap;
use std::io::{BufRead, Read, Result, Write};
use std::str::FromStr;
use std::{io::BufReader, net::TcpStream, time::Duration};

use ligma::get;

use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::method_verb::HttpMethod;
use crate::uri_params::{Query, Route};

const CONTENT_LENGTH_HEADER: &str = "content-length:";
const STREAM_READ_TIMEOUT: u64 = 5;

pub type HttpRequestHandler = Box<dyn Fn(Route, Query) -> HttpResponse>;

pub fn return_response(mut stream: std::net::TcpStream, response: HttpResponse) -> () {
    let status_description = response.status.get_status_info();
    let mut response_message = format!(
        "HTTP/1.1 {} {}\n\n",
        status_description.status_code, status_description.status_text
    );

    if response.body.is_some() {
        response_message = format!("{response_message}{}", response.body.unwrap());
    }

    stream
        .write_all(response_message.as_bytes())
        .unwrap_or_else(|e| println!("{e}"));
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

    let verb = HttpMethod::from_str(verb);

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

pub fn parse_query(uri: &str) -> Query {
    let query_start = uri.find('?');

    if query_start.is_none() {
        return Query(HashMap::new());
    }

    let queries_pairs = uri.get(query_start.unwrap() + 1..).unwrap();
    let mut queries = HashMap::new();

    for pair in queries_pairs.split('&') {
        let mut key_value = pair.split('=');
        let key = key_value.next().unwrap_or("");
        let value = key_value.next().unwrap_or("");

        if key != "" && value != "" {
            queries.insert(key.to_string(), value.to_string());
        }
    }

    Query(queries)
}

pub fn parse_route<'a>(
    endpoints: &'a HashMap<String, HttpRequestHandler>,
    request_uri: &str,
) -> (Option<&'a HttpRequestHandler>, Route) {
    let query_start = request_uri.find('?');

    let route = match query_start {
        Some(idx) => request_uri.get(0..idx).unwrap(),
        None => request_uri,
    };

    if let Some(handler) = endpoints.get(route) {
        return (Some(handler), Route(HashMap::new()));
    }

    let request_parts: Vec<_> = route.split('/').collect();
    let mut params: HashMap<String, String> = HashMap::new();
    let mut handler: Option<&HttpRequestHandler> = None;

    for key in endpoints.keys() {
        let endpoint_parts: Vec<_> = key.split('/').collect();

        if request_parts.len() != endpoint_parts.len() {
            continue;
        }

        handler = endpoints.get(key);

        for (i, &part) in endpoint_parts.iter().enumerate() {
            if part == request_parts[i] {
                continue;
            }

            if part.len() > 0
                && part.chars().nth(0).unwrap() == '{'
                && part.chars().nth(part.len() - 1).unwrap() == '}'
            {
                params.insert(
                    part.get(1..part.len() - 1).unwrap().to_string(),
                    request_parts[i].to_string(),
                );
            } else {
                params.clear();
                handler = None;
                break;
            }
        }
    }

    (handler, Route(params))
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

fn could_have_body(method: &HttpMethod) -> bool {
    match method {
        HttpMethod::Get | HttpMethod::Delete => false,
        HttpMethod::Post | HttpMethod::Put => true,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        http_response::HttpResponse,
        request_parser::{parse_route, HttpRequestHandler},
        uri_params::{Query, Route},
    };

    use super::parse_query;

    #[test]
    fn parse_query_params_empty_list() {
        let query = parse_query("http://localhost:42069/");

        assert_eq!(query, Query(HashMap::new()));
    }

    #[test]
    fn parse_query_params_not_empty_list() {
        let query = parse_query(
            "http://localhost:42069/some/path?a_param=123&b_param=str&c_param=[123,abc,123]",
        );

        assert_eq!(
            query,
            Query(HashMap::from([
                ("a_param".to_string(), "123".to_string()),
                ("b_param".to_string(), "str".to_string()),
                ("c_param".to_string(), "[123,abc,123]".to_string())
            ]))
        );
    }

    #[test]
    fn parse_query_params_incorrect_query() {
        let query = parse_query("/some/path?a_param=123&b_param=");

        assert_eq!(
            query,
            Query(HashMap::from([("a_param".to_string(), "123".to_string()),]))
        );
    }

    #[test]
    fn parse_route_params_empty_list() {
        let expected_handler: HttpRequestHandler = Box::new(|_, _| HttpResponse::ok(None));
        let handler_1: HttpRequestHandler = Box::new(|_, _| HttpResponse::ok(None));
        let handler_2: HttpRequestHandler = Box::new(|_, _| HttpResponse::ok(None));

        let handlers = &HashMap::from([
            ("/some/very/very/very/long/path".to_string(), handler_1),
            ("/some/very/very/long/path".to_string(), expected_handler),
            ("/some/very/long/path".to_string(), handler_2),
        ]);

        let (handler, route) =
            parse_route(handlers, "/some/very/very/long/path?a_param=123&b_param=");

        assert_eq!(route, Route(HashMap::new()));
        assert!(&handler.is_some());
    }

    #[test]
    fn parse_route_params_not_empty_list() {
        let expected_handler: HttpRequestHandler = Box::new(|_, _| HttpResponse::ok(None));
        let handler_1: HttpRequestHandler = Box::new(|_, _| HttpResponse::ok(None));
        let handler_2: HttpRequestHandler = Box::new(|_, _| HttpResponse::ok(None));

        let handlers = &HashMap::from([
            (
                "/some/very/{param_1}/very/{param_2}/path".to_string(),
                handler_1,
            ),
            ("/some/very/long/path".to_string(), handler_2),
            (
                "/some/{param_1}/very/{param_2}/{param_3}".to_string(),
                expected_handler,
            ),
        ]);

        let (handler, route) =
            parse_route(handlers, "/some/very/very/long/path?a_param=123&b_param=");

        let Route(actual_route_map) = route;
        let mut actual_route_vec: Vec<_> = actual_route_map.iter().collect();
        actual_route_vec.sort_by(|(a, _), (b, _)| a.cmp(b));

        assert_eq!(
            actual_route_vec,
            vec!(
                (&"param_1".to_string(), &"very".to_string()),
                (&"param_2".to_string(), &"long".to_string()),
                (&"param_3".to_string(), &"path".to_string()),
            )
        );
        assert!(&handler.is_some());
    }
}
