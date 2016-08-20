use rustc_serialize::json::{Object, Json, ToJson, encode};
use base::config::Config;
use base::db::MyPool;
use iron::prelude::*;
use persistent::Read;
use hbsi::Template;
use iron::status;
use mysql as my;
use iron_login::User;
use iron::Url;
use iron::modifiers::Redirect;
use iron::Handler;
use iron::modifiers::Header;
use hyper::header::Connection;
use router::NoRoute;

pub struct ResponseData(Object);

impl ResponseData {
    pub fn new(req: &mut Request) -> ResponseData {
        let config = req.get::<Read<Config>>().unwrap();
        let mut data = Object::new();
        data.insert("static_path".to_string(),
                    config.get("static_path").as_str().unwrap().to_string().to_json());
        let login = LoginUser::get_login(req);
        let raw_user = login.get_user();
        let mut login_user = Json::Null;
        if let Some(login_u) = raw_user {
            login_user = login_u.to_json();
        }
        data.insert("login_user".to_owned(), login_user);
        ResponseData(data)
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, key: &str, value: Json) -> &mut Self {
        self.0.insert(key.to_owned(), value);
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
    Redirect,
}

impl ToJson for JsonStatus {
    fn to_json(&self) -> Json {
        match *self {
            JsonStatus::Ok => Json::U64(0),
            JsonStatus::Fail => Json::U64(1),
            JsonStatus::Redirect => Json::U64(302),
        }
    }
}

pub struct JsonResponse {
    status: JsonStatus,
    message: String,
    data: Object,
    redirect_url: String,
}

impl ToJson for JsonResponse {
    fn to_json(&self) -> Json {
        let mut data = Object::new();
        data.insert("status".to_string(), self.status.to_json());
        data.insert("message".to_string(), self.message.to_json());
        data.insert("data".to_string(), self.data.to_json());
        data.insert("redirect_url".to_string(), self.redirect_url.to_json());
        data.to_json()
    }
}

pub fn temp_response(temp_name: &str, data: &ResponseData) -> IronResult<Response> {
    let mut resp = Response::new();
    resp.set_mut(Template::new(temp_name, data)).set_mut(status::Ok);
    Ok(resp)
}

pub fn not_found_response() -> IronResult<Response> {
    Err(IronError::new(NoRoute, status::NotFound))
}

pub fn json_response(status: JsonStatus, message: &str, data: Object, redirect_url: &str) -> IronResult<Response> {
    let mut resp = Response::new();
    let json_response = JsonResponse {
        status: status,
        message: message.to_owned(),
        data: data,
        redirect_url: redirect_url.to_owned(),
    };

    resp.set_mut(mime!(Application / Json)).set_mut(encode(&json_response.to_json()).unwrap()).set_mut(status::Ok);
    Ok(resp)
}

pub fn json_ok_response() -> IronResult<Response> {
    json_response(JsonStatus::Ok, "", Object::new(), "")
}

pub fn json_error_response(message: &str) -> IronResult<Response> {
    json_response(JsonStatus::Fail, message, Object::new(), "")
}

pub fn json_redirect_response(redirect_url: &str) -> IronResult<Response> {
    json_response(JsonStatus::Redirect, "", Object::new(), redirect_url)
}

#[derive(Debug, Clone)]
pub struct LoginUser {
    pub id: u64,
    pub username: String,
    pub email: String,
}

impl LoginUser {
    pub fn new(user_id: u64, username: &str, email: &str) -> LoginUser {
        LoginUser {
            id: user_id,
            username: username.to_owned(),
            email: email.to_owned(),
        }
    }
}

impl ToJson for LoginUser {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("username".to_owned(), self.username.to_json());
        object.insert("email".to_owned(), self.email.to_json());
        object.to_json()
    }
}

impl User for LoginUser {
    fn from_user_id(req: &mut Request, user_id: &str) -> Option<LoginUser> {
        let user_id = match user_id.parse::<u64>() {
            Ok(u) => u,
            _ => return None,
        };
        let pool = req.get::<Read<MyPool>>().unwrap().value();
        let mut result = pool.prep_exec("SELECT id, username, email from user where id=?",
                       (&user_id,))
            .unwrap();
        let row = result.next().unwrap().unwrap();
        let (id, username, email) = my::from_row::<(u64, String, String)>(row);
        Some(LoginUser::new(id, &username, &email))
    }

    fn get_user_id(&self) -> String {
        self.id.to_string()
    }
}

pub fn user_required<F>(handler: F) -> Box<Handler>
    where F: Send + Sync + 'static + Fn(&mut Request) -> IronResult<Response>
{

    let new_fn = move |req: &mut Request| -> IronResult<Response> {
        let login = LoginUser::get_login(req);
        let user = login.get_user();
        if user.is_none() {
            let config = req.get::<Read<Config>>().unwrap();
            let app_path = config.get("app_path").as_str().unwrap().to_owned();
            let url_str = app_path + "/user/login";
            if req.headers.get_raw("X-Requested-With").is_some() {
                let mut resp = json_redirect_response(&url_str).unwrap();
                resp.set_mut(Header(Connection::close()));
                return Ok(resp);
            }
            let url = Url::parse(&url_str).unwrap();
            return Ok(Response::with((status::Found, Redirect(url.clone()))));
        }
        handler(req)
    };

    Box::new(new_fn)
}
