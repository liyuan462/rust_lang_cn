use iron::prelude::*;
use base::framework::{ResponseData, temp_response};
use base::framework::LoginUser;
use iron_login::User;

pub fn index(req: &mut Request) -> IronResult<Response> {
    let data = ResponseData::new(req);
    temp_response("index", &data)
}
