extern crate hyper;
extern crate iron;
extern crate router;
extern crate mustache;

mod base;
mod handlers;
mod route;

fn main() {
    iron::Iron::new(route::gen_router()).http("localhost:3000").unwrap();
}
