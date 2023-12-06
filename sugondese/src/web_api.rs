use std::{
    collections::HashMap,
    io::Result,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    http_handler_info::HttpHandlerInfo,
    http_response::{HttpResponse, HttpStatus},
    method_verb::HttpMethod,
    request_parser::{handle_request, parse_request, return_response, HttpRequestHandler},
};

pub struct WebApi<'a> {
    addr: &'a str,
    threads_num: usize,
    endpoints: Endpoints,
}

#[derive(Clone)]
struct Endpoints {
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
            endpoints: Endpoints {
                get_endpoints: HashMap::new(),
                post_endpoints: HashMap::new(),
                delete_endpoints: HashMap::new(),
                put_endpoints: HashMap::new(),
            },
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let tcp_listener: Arc<Mutex<TcpListener>> =
            Arc::new(Mutex::new(TcpListener::bind(self.addr)?));

        let endpoints = Arc::new(self.endpoints.clone());
        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        for i in 1..self.threads_num {
            let listener = Arc::clone(&tcp_listener);
            let thread_endpoints = Arc::clone(&endpoints);

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

                let endpoints_map = get_endpoints_map(&request.method, &thread_endpoints);
                handle_request(request, endpoints_map, stream);
            });

            threads.push(handle);
        }

        for th in threads {
            th.join().unwrap();
        }

        Ok(())
    }

    pub fn get<Handler>(mut self, get_handler_info: Handler) -> Self
    where
        Handler: Fn() -> HttpHandlerInfo,
    {
        let handler_info = get_handler_info();
        let _ = &self
            .endpoints
            .get_endpoints
            .insert(handler_info.route, handler_info.handler);
        self
    }

    pub fn post<Handler>(mut self, get_handler_info: Handler) -> Self
    where
        Handler: Fn() -> HttpHandlerInfo,
    {
        let handler_info = get_handler_info();
        let _ = &self
            .endpoints
            .post_endpoints
            .insert(handler_info.route, handler_info.handler);
        self
    }

    pub fn delete<Handler>(mut self, get_handler_info: Handler) -> Self
    where
        Handler: Fn() -> HttpHandlerInfo,
    {
        let handler_info = get_handler_info();
        let _ = &self
            .endpoints
            .delete_endpoints
            .insert(handler_info.route, handler_info.handler);
        self
    }

    pub fn put<Handler>(mut self, get_handler_info: Handler) -> Self
    where
        Handler: Fn() -> HttpHandlerInfo,
    {
        let handler_info = get_handler_info();
        let _ = &self
            .endpoints
            .put_endpoints
            .insert(handler_info.route, handler_info.handler);
        self
    }
}

fn get_endpoints_map<'a>(
    method: &HttpMethod,
    endpoints: &'a Endpoints,
) -> &'a HashMap<String, HttpRequestHandler> {
    match method {
        HttpMethod::Get => &endpoints.get_endpoints,
        HttpMethod::Post => &endpoints.post_endpoints,
        HttpMethod::Delete => &endpoints.delete_endpoints,
        HttpMethod::Put => &endpoints.put_endpoints,
    }
}
