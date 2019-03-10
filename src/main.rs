extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::Read;
use std::thread;

fn main() {
    let mut router = Router::new();
    router.get("/", handler, "index");
    router.post("/register", register, "register");

    println!("fffffirst here");
    let listening = Iron::new(router).http("localhost:3000").unwrap();
    println!("here now!");

    fn handler(req: &mut Request) -> IronResult<Response> {
        println!("hit main handler");
        let ref query = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("/");
        Ok(Response::with((status::Ok, *query)))
    }

    fn register(req: &mut Request) -> IronResult<Response> {
        println!("hit register handler");
        let mut s = String::new();
        let len = req.body.read_to_string(&mut s);

        thread::spawn(move || {
            let mut router = Router::new();
            router.get("/new_endpoint", new_endpoint_handler, "new_endpoint");
            let listening_pt_2 = Iron::new(router).http("localhost:3001").unwrap();

            fn new_endpoint_handler(req: &mut Request) -> IronResult<Response> {
                println!("I am inside new endpoint handler");
                Ok(Response::with((status::Ok, "yes this is new endpoint")))
            }
        });

        println!("I should be going next");
        Ok(Response::with((status::Ok, format!("registered new handler: {:?}", req))))
    }
}