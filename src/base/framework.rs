use rustc_serialize::json::{Object, Json, ToJson, encode};
use base::config::Config;
use iron::prelude::*;
use persistent::Read;
use hbsi::Template;
use iron::status;
use mysql as my;
use iron_login::User;

pub struct ResponseData(Object);

impl ResponseData {
    pub fn new(req: &mut Request) -> ResponseData {
        let config = req.get::<Read<Config>>().unwrap();
        let mut data = Object::new();
        data.insert("static_path".to_string(),
                    config.get("static_path").as_str().unwrap().to_string().to_json());
        let login = LoginUser::get_login(req);
        let raw_user = login.get_user();
        let mut username = Json::Null;
        if let Some(LoginUser(name)) = raw_user {
            username = Json::String(name)
        }
        data.insert("login_user".to_string(), username);
        ResponseData(data)
    }

    #[allow(dead_code)]
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

pub enum JsonStatus {
    Ok,
    Fail,
}

impl ToJson for JsonStatus {
    fn to_json(&self) -> Json {
        match *self {
            JsonStatus::Ok => Json::U64(0),
            JsonStatus::Fail => Json::U64(1),
        }
    }
}

pub struct JsonResponse {
    status: JsonStatus,
    message: String,
    data: Object,
}

impl ToJson for JsonResponse {
    fn to_json(&self) -> Json {
        let mut data = Object::new();
        data.insert("status".to_string(), self.status.to_json());
        data.insert("message".to_string(), self.message.to_json());
        data.insert("data".to_string(), self.data.to_json());
        data.to_json()
    }
}

pub fn temp_response(temp_name: &str, data: &ResponseData) -> IronResult<Response> {
    let mut resp = Response::new();
    resp.set_mut(Template::new(temp_name, data)).set_mut(status::Ok);
    Ok(resp)
}

pub fn json_response(status: JsonStatus, message: &str, data: Object) -> IronResult<Response> {
    let mut resp = Response::new();
    let json_response = JsonResponse {
        status: status,
        message: message.to_string(),
        data: data,
    };

    resp.set_mut(mime!(Application/Json)).set_mut(encode(&json_response.to_json()).unwrap()).set_mut(status::Ok);
    Ok(resp)
}

pub fn json_ok_response() -> IronResult<Response> {
    json_response(JsonStatus::Ok, "", Object::new())
}

pub fn json_error_response(message: &str) -> IronResult<Response> {
    json_response(JsonStatus::Fail, message, Object::new())
}

#[derive(Debug)]
pub struct LoginUser(String);

impl LoginUser {
    pub fn new(user_id: &str) -> LoginUser {
        LoginUser(user_id.to_owned())
    }
}

impl User for LoginUser {
    fn from_user_id(_: &mut Request, user_id: &str) -> Option<LoginUser> {
        Some(LoginUser(user_id.to_owned()))
    }

    fn get_user_id(&self) -> String {
        self.0.to_owned()
    }
}
