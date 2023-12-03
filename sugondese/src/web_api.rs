use std::{
    collections::HashMap,
    io::Result,
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    http_handler_info::HttpHandlerInfo,
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
            let response = handler(route, query, request.body);

            return_response(stream, response);
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
            .put_endpoints
            .insert(handler_info.route, handler_info.handler);
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
