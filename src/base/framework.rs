use rustc_serialize::json::{Object, Json, ToJson};
use base::config::Config;
use iron::prelude::*;
use persistent::Read;
use hbsi::Template;
use iron::status;

pub struct ResponseData(Object);

impl ResponseData {
    pub fn new(req: &mut Request) -> ResponseData {
        let config = req.get::<Read<Config>>().unwrap();
        let mut data = Object::new();
        data.insert("static_path".to_string(),
                    config.get("static_path").as_str().unwrap().to_string().to_json());
        ResponseData(data)
    }

    pub fn insert(&mut self, key: String, value: Json) -> &mut Self {
        self.0.insert(key, value);
        self
    }
}

impl<'a> ToJson for &'a ResponseData {
    fn to_json(&self) -> Json {
        self.0.to_json()
    }
}

pub fn temp_response(temp_name: &str, data: &ResponseData) -> IronResult<Response> {
    let mut resp = Response::new();
    resp.set_mut(Template::new(temp_name, data)).set_mut(status::Ok);
    Ok(resp)
}
