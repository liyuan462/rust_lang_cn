extern crate hyper;
extern crate iron;
extern crate router;
extern crate handlebars_iron as hbsi;
extern crate rustc_serialize;
extern crate persistent;
extern crate urlencoded;
extern crate traitobject;
#[macro_use]
extern crate mime;
extern crate mysql;
extern crate crypto;
extern crate rand;
extern crate iron_login;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate pulldown_cmark;
extern crate regex;
extern crate ammonia;

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
    // init logging
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let mut chain = Chain::new(route::gen_router());
    let config = Config::new();
    chain.link_before(Read::<Config>::one(config.clone()));

    let my_pool = MyPool::new(&config);
    chain.link_before(Read::<MyPool>::one(my_pool));

    let cookie_sign_key = config.get("cookie_sign_key").as_str().unwrap().as_bytes().to_owned();
    chain.link_around(iron_login::LoginManager::new(cookie_sign_key));

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("templates/", ".hbs")));

    if let Err(r) = hbse.reload() {
        panic!("{}", r.description());
    }

    chain.link_after(hbse);

    iron::Iron::new(chain).http("0.0.0.0:3000").unwrap();
}
