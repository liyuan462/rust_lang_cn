use iron::prelude::*;
use base::framework::{ResponseData, temp_response,
                      json_error_response, json_ok_response, not_found_response};
use urlencoded::UrlEncodedBody;
use base::validator::{Validator, Checker, Str, StrValue,
                      Max, Min, Format, Optional, Int, IntValue};
use mysql as my;
use crypto::md5;
use crypto::digest::Digest;
use rand;
use rand::Rng;
use chrono::*;
use base::db::MyPool;
use persistent::Read;
use base::framework::LoginUser;
use iron_login::User as U;
use base::model::{User, Article, Category, Comment};
use router::Router;
use rustc_serialize::json::{Object, Json, ToJson};
use base::util::gen_gravatar_url;
use base::util::render_html;
use base::constant;
use oven::prelude::*;
use cookie::Cookie;
use time;

pub fn register_load(req: &mut Request) -> IronResult<Response> {
    let data = ResponseData::new(req);
    temp_response("user/register_load", &data)
}

pub fn register(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();

    validator
        .add_checker(
            Checker::new("username", Str, "用户名")
                << Min(3)
                << Max(32)
                << Format(r"^[a-zA-Z_][\da-zA-Z_]{2,}$"))
        .add_checker(
            Checker::new("email", Str, "邮箱")
                << Min(5)
                << Max(64)
                << Format(r"^[^@]+@[^@]+\.[^@]+$"))
        .add_checker(
            Checker::new("password", Str, "密码")
                << Min(8)
                << Max(32));

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
    let mut stmt = pool.prepare(
        "INSERT INTO user(username, email, \
         password, salt, create_time) VALUES (?, ?, ?, ?, ?)")
        .unwrap();
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
        .add_checker(Checker::new("password", Str, "密码"))
        .add_checker(Checker::new("persist", Int, "记住登录状态") << Optional);

    validator.validate(req.get::<UrlEncodedBody>());
    if !validator.is_valid() {
        return json_error_response(&validator.messages[0]);
    }

    let username = validator.get_valid::<StrValue>("username").value();
    let password = validator.get_valid::<StrValue>("password").value();

    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let mut result = pool.prep_exec(
        "SELECT id, email, password, salt from user where username=?",
        (&username,)).unwrap();
    let raw_row = result.next();
    if raw_row.is_none() {
        return json_error_response("对不起，用户名或密码不对");
    }
    let row = raw_row.unwrap().unwrap();
    let (user_id, email, pass, salt) = my::from_row::<(
        u64, String, String, String)>(row);
    let password_with_salt = password + &salt;
    let mut sh = md5::Md5::new();
    sh.input_str(&password_with_salt);
    let hash = sh.result_str();
    if pass != hash {
        return json_error_response("对不起，用户名或密码不对");
    }

    // set session
    let mut resp = json_ok_response().unwrap();
    let mut c = Cookie::new("logged_in_user".to_owned(), "".to_owned());
    c.httponly = true;
    if match validator.valid_data.get("persist") {
        Some(p) => if p[0].downcast_ref_unchecked::<IntValue>().value() == 1 {
            true} else {false},
        _ => false
    } {
        c.expires = Some(time::now() + time::Duration::days(365));
    }
    c.path = Some("/".to_owned());
    c.value = LoginUser::new(user_id, &username, &email).get_user_id();
    resp.set_cookie(c);
    Ok(resp)
}

pub fn logout(req: &mut Request) -> IronResult<Response> {
    let login = LoginUser::get_login(req);
    let mut resp = json_ok_response().unwrap();
    resp.set_mut(login.log_out());
    Ok(resp)
}

pub fn show(req: &mut Request) -> IronResult<Response> {
    let user_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("user_id").unwrap()
                       .parse::<u64>().map_err(|_| not_found_response().unwrap_err()));

    let login_user = LoginUser::get_login(req).get_user();

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut data = ResponseData::new(req);

    if get_general_info(&mut data, &pool, user_id, login_user.clone()).is_err() {
        return not_found_response();
    }

    get_unread_messages_count(&mut data, &pool, user_id, login_user);

    // get articles
    let articles: Vec<Article> = pool.prep_exec(
        "SELECT id, category, title, content, comments_count, \
         create_time from article where status=? and user_id=? order by \
         create_time desc",
        (constant::ARTICLE::STATUS::NORMAL, user_id))
        .unwrap()
        .map(|x| x.unwrap())
        .map(|row| {
            let (id, category, title, content,
                 comments_count, create_time) = my::from_row(row);

            Article {
                id: id,
                category: Category::from_value(category),
                title: title,
                content: content,
                comments_count: comments_count,
                user: User::default(),
                create_time: create_time,
                update_time: *constant::DEFAULT_DATETIME,
                flag: 0,
                comments: Vec::new(),
            }
        }).collect();

    data.insert("articles", articles.to_json());
    data.insert("articles_active", true.to_json());
    temp_response("user/show", &data)
}

pub fn show_comments(req: &mut Request) -> IronResult<Response> {
    let user_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("user_id").unwrap()
                       .parse::<u64>().map_err(|_| not_found_response().unwrap_err()));

    let login_user = LoginUser::get_login(req).get_user();

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut data = ResponseData::new(req);

    if get_general_info(&mut data, &pool, user_id, login_user.clone()).is_err() {
        return not_found_response();
    }

    get_unread_messages_count(&mut data, &pool, user_id, login_user);

    // get comments
    let comments: Vec<Comment> = pool.prep_exec(
        "SELECT c.id, c.content, c.create_time, \
         a.id as article_id, a.title as article_title from comment as c \
         join article as a on c.article_id=a.id where c.user_id=? \
         order by c.create_time desc",
        (user_id,))
        .unwrap()
        .map(|x| x.unwrap())
        .map(|row| {
            let (id, content, create_time,
                 article_id, article_title) = my::from_row(row);

            let mut article = Article::default();
            article.id = article_id;
            article.title = article_title;

            let mut comment = Comment {
                id: id,
                content: content,
                user: User:: default(),
                create_time: create_time,
                article: Some(article),
            };

            comment.content = render_html(&comment.content);
            comment
        }).collect();

    data.insert("comments", comments.to_json());
    data.insert("comments_active", true.to_json());
    temp_response("user/show", &data)
}

pub fn show_messages(req: &mut Request) -> IronResult<Response> {
    let user_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("user_id").unwrap()
                       .parse::<u64>().map_err(|_| not_found_response().unwrap_err()));

    let login_user = LoginUser::get_login(req).get_user();

    if login_user.clone().unwrap().id != user_id {
        return not_found_response();
    }

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut data = ResponseData::new(req);

    if get_general_info(&mut data, &pool, user_id, login_user).is_err() {
        return not_found_response();
    }

    // get messages
    let messages: Vec<Json> = pool.prep_exec(
        "SELECT m.status, m.create_time, c.content, u.id as user_id, u.username, \
         u.email, a.id as article_id, a.title as article_title \
         from message as m join user as u on m.from_user_id=u.id \
         join article as a on a.id=m.article_id \
         join comment as c on c.id=m.comment_id \
         where to_user_id=? order by create_time desc",
        (user_id,))
        .unwrap()
        .map(|x| x.unwrap())
        .map(|row| {
            let (status, create_time, content, user_id, username, email,
                 article_id, article_title)
                = my::from_row::<(
                    i8, NaiveDateTime, String, u64,
                    String, String, u64, String)>(row);

            let mut object = Object::new();
            object.insert("is_new".to_owned(),
                          (if status == constant::MESSAGE::STATUS::INIT {true}
                           else {false}).to_json());
            object.insert("create_time".to_owned(), create_time.format(
                "%Y-%m-%d %H:%M:%S").to_string().to_json());
            object.insert("content".to_owned(), render_html(&content).to_json());
            object.insert("user_id".to_owned(), user_id.to_json());
            object.insert("username".to_owned(), username.to_json());
            object.insert("avatar".to_owned(), gen_gravatar_url(&email).to_json());
            object.insert("article_id".to_owned(), article_id.to_json());
            object.insert("article_title".to_owned(), article_title.to_json());
            object.to_json()
        }).collect();

    // mark messages as read
    pool.prep_exec("UPDATE message set status=? where to_user_id=? and status=?",
                   (constant::MESSAGE::STATUS::READ,
                    user_id,
                    constant::MESSAGE::STATUS::INIT)).unwrap();

    data.insert("messages", messages.to_json());
    data.insert("messages_active", true.to_json());
    temp_response("user/show", &data)
}

fn get_general_info(data: &mut ResponseData,
                    pool: &my::Pool,
                    user_id: u64,
                    raw_login_user: Option<LoginUser>) -> Result<(),()> {
    let row = try!(pool.prep_exec(
        "SELECT id, username, email, create_time from user where id=?", (&user_id,))
                   .unwrap()
                   .next()
                   .map(|row|row.unwrap()).ok_or(()));

    let (user_id, username, email, create_time) = my::from_row::<(_,_,String,_)>(row);
    let user = User{
        id: user_id,
        avatar: gen_gravatar_url(&email),
        username: username,
        email: email,
        create_time: create_time,
    };

    // get articles count
    let articles_count = my::from_row::<usize>(
        pool.prep_exec("SELECT count(id) from article where status=? and user_id=?",
                       (constant::ARTICLE::STATUS::NORMAL, user_id))
            .unwrap().next().unwrap().unwrap());

    // get comments count
    let comments_count = my::from_row::<usize>(
        pool.prep_exec("SELECT count(id) from comment where user_id=?",
                       (user_id,))
        .unwrap().next().unwrap().unwrap());

    // where is me among all members
    let which_member = my::from_row::<usize>(
        pool.prep_exec("SELECT count(id) as count from user where id < ?",
                       (user_id,))
            .unwrap().next().unwrap().unwrap()) + 1;

    // judge whether is myself
    let mut is_myself = false;
    if let Some(login_user) = raw_login_user {
        if login_user.id == user.id {
            is_myself = true;
        }
    }

    data.insert("user", user.to_json());
    // user register date
    data.insert("register_date",
                user.create_time.format("%Y-%m-%d").to_string().to_json());
    data.insert("which_member", which_member.to_json());
    data.insert("articles_count", articles_count.to_json());
    data.insert("comments_count", comments_count.to_json());
    data.insert("is_myself", is_myself.to_json());
    Ok(())
}

fn get_unread_messages_count(data: &mut ResponseData, pool: &my::Pool,
                             user_id: u64, raw_login_user: Option<LoginUser>) {

    if raw_login_user.is_none() {
        return;
    }

    let login_user = raw_login_user.unwrap();

    if login_user.id != user_id {
        return;
    }

    let unread_messages_count = my::from_row::<usize>(pool.prep_exec(
        "SELECT count(id) as count from message where to_user_id=? and status=?",
        (login_user.id, constant::MESSAGE::STATUS::INIT)).unwrap()
                                                    .next().unwrap().unwrap());
    data.insert("unread_messages_count", unread_messages_count.to_json());
}
