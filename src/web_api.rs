use std::{
    collections::HashMap,
    io::Result,
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    http_request::HttpRequest,
    request_parser::{
        parse_query, parse_request, parse_route, return_bad_request, return_not_found,
        return_ok_response, HttpRequestHandler,
    },
};

pub struct WebApi<'a> {
    addr: &'a str,
    threads_num: usize,
    get_endpoints: HashMap<String, HttpRequestHandler>,
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
                    return_bad_request(stream);
                    continue;
                }
                let request = request.unwrap();

                tx.send((request, stream))
                    .unwrap_or_else(|e| println!("{e}"));
            });

            threads.push(handle);
        }

        for (request, stream) in rx.iter() {
            let (handler, route) = parse_route(&self.get_endpoints, &request.uri);

            if handler.is_none() {
                return_not_found(stream);
                continue;
            }

            let handler = handler.unwrap();
            let query = parse_query(&request.uri);
            let response = handler(route, query);

            return_ok_response(stream, &response);
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
}

#[cfg(test)]
mod tests {
    use super::WebApi;
    use crate::uri_params::{Query, Route};

    fn hello_handler(_route_params: Route, _query_params: Query) -> String {
        "hello from handler".to_string()
    }

    fn route_params_handler(_route_params: Route, _query_params: Query) -> String {
        "hello from route params handler".to_string()
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
            .run();
    }
}
