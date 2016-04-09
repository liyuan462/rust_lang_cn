extern crate iron;
extern crate router;
extern crate mustache;

mod base;

use iron::prelude::*;
use router::Router;
use mustache::MapBuilder;

use base::template_response;


fn main() {
    let mut router = Router::new();
    router.get("/", index);

    Iron::new(router).http("localhost:3000").unwrap();
}


fn index(_: &mut Request) -> IronResult<Response> {
    let data = MapBuilder::new()
        .insert_str("name", "Rust China!")
        .build();

    template_response("index.html", data)
}
