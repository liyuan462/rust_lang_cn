use iron::prelude::*;
use base::framework::{ResponseData, temp_response,
                      json_error_response, json_ok_response, not_found_response};
use urlencoded::UrlEncodedBody;
use base::validator::{Validator, Checker, Str, StrValue, Max, Min, Format};
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
use rustc_serialize::json::ToJson;
use base::util::gen_gravatar_url;
use base::util::render_html;
use base::constant;

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
        .add_checker(Checker::new("password", Str, "密码"));

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

pub fn show(req: &mut Request) -> IronResult<Response> {
    let user_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("user_id").unwrap()
                       .parse::<u64>().map_err(|_| not_found_response().unwrap_err()));

    let login_user = LoginUser::get_login(req).get_user();

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut data = ResponseData::new(req);

    if get_general_info(&mut data, &pool, user_id, login_user).is_err() {
        return not_found_response();
    }

    // get articles
    let articles: Vec<Article> = pool.prep_exec(
        "SELECT id, category, title, content, comments_count, \
         create_time from article where status=? and user_id=? order by \
         create_time desc",
        (constant::ARTICLE_STATUS::NORMAL, user_id))
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

    if get_general_info(&mut data, &pool, user_id, login_user).is_err() {
        return not_found_response();
    }

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
                       (constant::ARTICLE_STATUS::NORMAL, user_id))
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
