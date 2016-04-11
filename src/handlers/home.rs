use iron::prelude::*;
use hbsi::Template;
use iron::status;
use rustc_serialize::json::ToJson;
use base::framework::{ResponseData, temp_response};

pub fn index(req: &mut Request) -> IronResult<Response> {
    let mut data = ResponseData::new(req);
    data.insert("name".to_string(), "你好!".to_string().to_json());
    temp_response("index", &data)
}
