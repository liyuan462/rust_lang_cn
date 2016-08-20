extern crate toml;

use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use self::toml::{Table, Value, Parser};
use std::cmp::Ord;
use std::borrow::Borrow;
use iron::typemap::Key;

#[derive(Clone)]
pub struct Config(Table);

impl Config {
    pub fn new() -> Config {
        let path = Path::new("config.toml");
        let mut file = File::open(&path).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        let value = Parser::new(&s).parse().unwrap();
        Config(value)
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> &Value
        where String: Borrow<Q>,
              Q: Ord
    {
        self.0.get(key).unwrap()
    }

    pub fn value(&self) -> Table {
        self.0.clone()
    }
}

impl Key for Config {
    type Value = Config;
}
