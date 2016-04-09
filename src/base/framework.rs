use std::path::Path;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use iron::prelude::*;
use iron::modifiers::Header;
use iron::status;


pub fn template_response(template_name: &str, data: ::mustache::Data) -> IronResult<Response>{
    let path_string = format!("{}{}", "templates/", template_name);
    let path = Path::new(&path_string);
    let template = ::mustache::compile_path(path).unwrap();
    let mut body: Vec<u8> = Vec::new();
    template.render_data(&mut body, &data);
    let content_type = ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![]));
    Ok(Response::with((status::Ok, body, Header(content_type))))
}
