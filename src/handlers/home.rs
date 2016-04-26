use iron::prelude::*;
use base::framework::{ResponseData, temp_response};

pub fn index(req: &mut Request) -> IronResult<Response> {
    let data = ResponseData::new(req);
    temp_response("index", &data)
}
