use iron::prelude::*;
use base::framework::{ResponseData, temp_response,
                      json_error_response, json_ok_response};
use urlencoded::UrlEncodedBody;
use base::validator::{Validator, Checker, Str, StrValue, Max, Min};
use mysql as my;
use crypto::md5;
use crypto::digest::Digest;
use rand;
use rand::Rng;
use chrono::*;
use base::db::MyPool;
use persistent::Read;
use base::framework::LoginUser;
use iron_login::User;

pub fn register_load(req: &mut Request) -> IronResult<Response> {
    let data = ResponseData::new(req);
    temp_response("user/register_load", &data)
}

pub fn register(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("username", Str, "用户名") << Min(3) << Max(32))
        .add_checker(Checker::new("email", Str, "邮箱") << Min(5) << Max(64))
        .add_checker(Checker::new("password", Str, "密码") << Min(8) << Max(32));

    validator.validate(req.get::<UrlEncodedBody>());
    if !validator.is_valid() {
        return json_error_response(&validator.messages[0]);
    }

    let username = validator.get_valid::<StrValue>("username").value();
    let email = validator.get_valid::<StrValue>("email").value();
    let password = validator.get_valid::<StrValue>("password").value();

    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let salt = rand::thread_rng()
        .gen_ascii_chars()
        .take(32)
        .collect::<String>();

    let now = Local::now().naive_local();
    let password_with_salt = password + &salt;
    let mut sh = md5::Md5::new();
    sh.input_str(&password_with_salt);
    let hash = sh.result_str();
    let mut stmt = pool.prepare(r"INSERT INTO user(username, email, password, salt, create_time) VALUES (?, ?, ?, ?, ?)").unwrap();
    let result = stmt.execute((username, email, hash, salt, now));
    if let Err(my::error::Error::MySqlError(ref e)) = result {
        if e.code == 1062 {
            return json_error_response("对不起，该用户已经被注册了");
        }
    }
    result.unwrap();
    json_ok_response()
}

pub fn login_load(req: &mut Request) -> IronResult<Response> {
    let data = ResponseData::new(req);
    temp_response("user/login_load", &data)
}

pub fn login(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("username", Str, "用户名"))
        .add_checker(Checker::new("password", Str, "密码"));

    validator.validate(req.get::<UrlEncodedBody>());
    if !validator.is_valid() {
        return json_error_response(&validator.messages[0]);
    }

    let username = validator.get_valid::<StrValue>("username").value();
    let password = validator.get_valid::<StrValue>("password").value();

    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let mut result = pool.prep_exec("SELECT id, email, password, salt from user where username=?", (&username,)).unwrap();
    let raw_row = result.next();
    if raw_row.is_none() {
        return json_error_response("对不起，用户名或密码不对");
    }
    let row = raw_row.unwrap().unwrap();
    let (user_id, email, pass, salt) = my::from_row::<(u64, String, String, String)>(row);
    let password_with_salt = password + &salt;
    let mut sh = md5::Md5::new();
    sh.input_str(&password_with_salt);
    let hash = sh.result_str();
    if pass != hash {
        return json_error_response("对不起，用户名或密码不对");
    }

    // set session
    let login = LoginUser::get_login(req);
    let mut resp = json_ok_response().unwrap();
    resp.set_mut(login.log_in(LoginUser::new(user_id, &username, &email)));
    Ok(resp)
}

pub fn logout(req: &mut Request) -> IronResult<Response> {
    let login = LoginUser::get_login(req);
    let mut resp = json_ok_response().unwrap();
    resp.set_mut(login.log_out());
    Ok(resp)
}
