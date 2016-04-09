use iron::prelude::*;
use mustache::MapBuilder;
use base::framework::template_response;

pub fn index(_: &mut Request) -> IronResult<Response> {
    let data = MapBuilder::new()
        .insert_str("name", "Rust China!")
        .build();

    template_response("index.html", data)
}
