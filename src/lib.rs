use std::{
    collections::HashMap,
    io::Result,
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use http_request::HttpRequest;
use request_parser::{return_not_found, return_ok_response};

use crate::request_parser::{parse_request, return_bad_request};

mod http_request;
mod method_verb;
mod request_parser;

pub struct WebApi<'a> {
    addr: &'a str,
    threads_num: usize,
    get_endpoints: HashMap<String, Box<dyn Fn(Route, Query) -> &'a str>>,
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
            let handler = self.get_endpoints.get(&request.uri);

            if handler.is_none() {
                return_not_found(stream);
                continue;
            }

            let handler = handler.unwrap();
            let response = handler(Route(HashMap::new()), Query(HashMap::new()));

            return_ok_response(stream, response);
        }

        for th in threads {
            th.join().unwrap();
        }

        Ok(())
    }

    pub fn get(
        mut self,
        route: &'a str,
        handler: Box<dyn Fn(Route, Query) -> &'a str + 'static>,
    ) -> Self {
        let _ = &self.get_endpoints.insert(route.to_string(), handler);

        self
    }
}

pub struct Route(HashMap<String, String>);
pub struct Query(HashMap<String, String>);
