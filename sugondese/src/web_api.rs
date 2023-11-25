use std::{
    collections::HashMap,
    io::Result,
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    http_request::HttpRequest,
    http_response::{HttpResponse, HttpStatus},
    method_verb::HttpMethod,
    request_parser::{
        parse_query, parse_request, parse_route, return_response, HttpRequestHandler,
    },
};

pub struct WebApi<'a> {
    addr: &'a str,
    threads_num: usize,
    get_endpoints: HashMap<String, HttpRequestHandler>,
    post_endpoints: HashMap<String, HttpRequestHandler>,
    delete_endpoints: HashMap<String, HttpRequestHandler>,
    put_endpoints: HashMap<String, HttpRequestHandler>,
}

impl<'a> WebApi<'a> {
    pub fn new(addr: &'a str, threads_num: usize) -> WebApi {
        if threads_num <= 0 {
            panic!("Threads number must be more than 0");
        }

        WebApi {
            addr,
            threads_num,
            get_endpoints: HashMap::new(),
            post_endpoints: HashMap::new(),
            delete_endpoints: HashMap::new(),
            put_endpoints: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let tcp_listener: Arc<Mutex<TcpListener>> =
            Arc::new(Mutex::new(TcpListener::bind(self.addr)?));

        let mut threads: Vec<JoinHandle<()>> = Vec::new();
        let (tx, rx) = mpsc::channel::<(HttpRequest, TcpStream)>();

        for i in 1..self.threads_num {
            let listener = Arc::clone(&tcp_listener);
            let tx = tx.clone();

            let handle = thread::spawn(move || loop {
                let stream = listener.lock().unwrap().incoming().next().unwrap().unwrap();

                println!("thread {i} handles request");

                let request = parse_request(stream.try_clone().unwrap());

                if request.is_none() {
                    return_response(
                        stream,
                        HttpResponse {
                            status: HttpStatus::BadRequest,
                            body: None,
                        },
                    );
                    continue;
                }
                let request = request.unwrap();

                tx.send((request, stream))
                    .unwrap_or_else(|e| println!("{e}"));
            });

            threads.push(handle);
        }

        for (request, stream) in rx.iter() {
            let endpoints_map = self.get_endpoints_map(request.method);
            let (handler, route) = parse_route(endpoints_map, &request.uri);

            if handler.is_none() {
                return_response(
                    stream,
                    HttpResponse {
                        status: HttpStatus::NotFound,
                        body: None,
                    },
                );
                continue;
            }

            let handler = handler.unwrap();
            let query = parse_query(&request.uri);
            let response = handler(route, query);

            return_response(stream, response);
        }

        for th in threads {
            th.join().unwrap();
        }

        Ok(())
    }

    pub fn get(mut self, route: &'a str, handler: HttpRequestHandler) -> Self {
        let _ = &self.get_endpoints.insert(route.to_string(), handler);
        self
    }

    pub fn post(mut self, route: &'a str, handler: HttpRequestHandler) -> Self {
        let _ = &self.post_endpoints.insert(route.to_string(), handler);
        self
    }

    pub fn delete(mut self, route: &'a str, handler: HttpRequestHandler) -> Self {
        let _ = &self.delete_endpoints.insert(route.to_string(), handler);
        self
    }

    pub fn put(mut self, route: &'a str, handler: HttpRequestHandler) -> Self {
        let _ = &self.put_endpoints.insert(route.to_string(), handler);
        self
    }

    fn get_endpoints_map(&self, method: HttpMethod) -> &HashMap<String, HttpRequestHandler> {
        match method {
            crate::method_verb::HttpMethod::Get => &self.get_endpoints,
            crate::method_verb::HttpMethod::Post => &self.post_endpoints,
            crate::method_verb::HttpMethod::Delete => &self.delete_endpoints,
            crate::method_verb::HttpMethod::Put => &self.put_endpoints,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ligma::http_handler;
    use serde::{Deserialize, Serialize};

    use super::WebApi;
    use crate::{
        http_response::HttpResponse,
        uri_params::{Query, Route},
    };

    fn hello_handler(_route_params: Route, _query_params: Query) -> HttpResponse {
        HttpResponse::ok(Some("hello from handler".to_string()))
    }

    fn route_params_handler(_route_params: Route, _query_params: Query) -> HttpResponse {
        HttpResponse::ok(Some("hello from route params handler".to_string()))
    }

    fn post_handler(_route_params: Route, _query_params: Query) -> HttpResponse {
        HttpResponse::ok(Some("hello from post method".to_string()))
    }

    #[test]
    #[ignore = "starts api"]
    fn aggr_result_struct_err() {
        let _ = WebApi::new("172.17.0.2:6080", 5)
            .get("/", Box::new(hello_handler))
            .get(
                "/route_params/{param_1}/{param_2}/hello",
                Box::new(route_params_handler),
            )
            .post("/", Box::new(post_handler))
            .run();
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        omega: u32,
    }

    #[http_handler]
    fn test_fn(route: Route, query: Query, body: TestStruct) -> &'static str {
        "hello from original function"
    }

    #[test]
    fn macro_test() {
        //test_fn_1(Route(HashMap::new()), Query(HashMap::new()));
        println!(
            "{}",
            test_fn(
                Route(HashMap::new()),
                Query(HashMap::new()),
                "{ \"omega\": 5 }",
            )
        );
    }
}
