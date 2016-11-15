#![feature(box_syntax)]

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_timer;
extern crate hyper;
extern crate route_recognizer;

use std::time::Duration;
use futures::{ Future, finished, lazy };
use futures_cpupool::CpuPool;
use tokio_timer::Timer;
use hyper::StatusCode;
use hyper::header::ContentLength;
use hyper::server::{Server, Request, Response};

pub mod host;

use host::*;

struct Echo {
    cpu_pool: CpuPool
}

impl Service for Echo {
    fn route(&self) -> &'static str { "/" }

    fn call(&self, _: Params, _: Request) -> HttpFuture {
        // Do some 'expensive work' on a background thread
        let work = self.cpu_pool
            .spawn(lazy(|| {
                Timer::default()
                    .sleep(Duration::from_millis(1000))
                    .and_then(|_| finished("Hello world".as_bytes()))
            }));

        let respond = work
            .then(|msg| {
                let response = match msg {
                    Ok(msg) => {
                        Response::new()
                            .header(ContentLength(msg.len() as u64))
                            .body(msg)
                    },
                    Err(_) => {
                        Response::new()
                            .status(StatusCode::InternalServerError)
                    }
                };

                finished(response)
            });

        box respond
    }
}

fn main() {
    let cpu_pool = CpuPool::new(4);

    let mut router = host::router::Router::new();
    router.get(Echo { cpu_pool: cpu_pool.clone() });

    let addr = "127.0.0.1:1337".parse().unwrap();
    let server = Server::http(&addr).unwrap();
    let (lst, server) = server.standalone(move || Ok(router.clone())).unwrap();

    println!("listening on {}", lst);

    server.run();
}
