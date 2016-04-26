use iron::prelude::*;
use base::framework::{ResponseData, temp_response,
                      json_error_response, json_ok_response,
                      get_db_pool};
use urlencoded::UrlEncodedBody;
use base::validator::{Validator, Checker, Str, StrValue, Int, Max, Min};
use std::any::Any;
use rustc_serialize::json::{Object, Json, ToJson, encode};
use mysql as my;
use mysql::error::Result as MyResult;
use crypto::md5;
use crypto::digest::Digest;
use rand;
use rand::Rng;
use time;
use base::db::MyPool;
use persistent::Read;

pub fn register_load(req: &mut Request) -> IronResult<Response> {
    let data = ResponseData::new(req);
    temp_response("user/register_load", &data)
}

pub fn register(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator.add_checker(Checker::new("email", Str, "邮箱") << Min(12) << Max(32))
        .add_checker(Checker::new("password", Str, "密码") << Min(12) << Max(32));
    validator.validate(req.get::<UrlEncodedBody>());
    if !validator.is_valid() {
        return json_error_response(&validator.messages[0]);
    }

    let email = validator.get_valid::<StrValue>("email").value();
    let password = validator.get_valid::<StrValue>("password").value();

    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let salt = rand::thread_rng()
        .gen_ascii_chars()
        .take(32)
        .collect::<String>();

    let now = format!("{}", time::now().strftime("%Y-%m-%d %H:%M:%S").unwrap());
    let password_with_salt = password + &salt;
    let mut sh = md5::Md5::new();
    sh.input_str(&password_with_salt);
    let hash = sh.result_str();
    let mut stmt = pool.prepare(r"INSERT INTO user(email, password, salt, create_time) VALUES (?, ?, ?, ?)").unwrap();
    let result = stmt.execute((email, hash, salt, now));
    if let Err(my::error::Error::MySqlError(ref e)) = result {
        if e.code == 1062 {
            return json_error_response("对不起，邮箱已经被注册了");
        }
    }
    result.unwrap();
    json_ok_response()
}
