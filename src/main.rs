extern crate hyper;
extern crate iron;
extern crate router;
extern crate handlebars_iron as hbsi;
extern crate rustc_serialize;
extern crate persistent;
extern crate urlencoded;
extern crate traitobject;
#[macro_use] extern crate mime;
extern crate mysql;
extern crate crypto;
extern crate rand;
extern crate time;

mod base;
mod handlers;
mod route;

use iron::Chain;
use hbsi::{HandlebarsEngine, DirectorySource};
use std::error::Error;
use persistent::Read;
use base::config::Config;
use base::db::MyPool;

fn main() {
    let mut chain = Chain::new(route::gen_router());

    let config = Config::new();
    chain.link_before(Read::<Config>::one(config.clone()));

    let myPool = MyPool::new(&config);
    chain.link_before(Read::<MyPool>::one(myPool));

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("templates/", ".hbs")));

    if let Err(r) = hbse.reload() {
        panic!("{}", r.description());
    }

    chain.link_after(hbse);
    iron::Iron::new(chain).http("localhost:3000").unwrap();
}
