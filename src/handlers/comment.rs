use iron::prelude::*;
use base::framework::{json_error_response, json_ok_response,
                      not_found_response};
use urlencoded::UrlEncodedBody;
use base::db::MyPool;
use base::validator::{Validator, Checker, Str, StrValue, Int, IntValue, Min};
use base::framework::LoginUser;
use iron_login::User as U;
use persistent::Read;
use chrono::*;
use mysql as my;
use regex::Regex;
use regex::Captures;
use base::config::Config;

pub fn new(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("article_id", Int, "文章ID") << Min(1))
        .add_checker(Checker::new("content", Str, "内容") << Min(7));

    validator.validate(req.get::<UrlEncodedBody>());
    if !validator.is_valid() {
        return json_error_response(&validator.messages[0]);
    }

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let article_id = validator.get_valid::<IntValue>("article_id").value() as u64;

    // check whether article exists
    if pool.prep_exec("SELECT id from article where id=?", (&article_id,)).unwrap().next().is_none() {
            return not_found_response();
    };

    let content = validator.get_valid::<StrValue>("content").value();
    let login = LoginUser::get_login(req);
    let user = login.get_user().unwrap();
    let now = Local::now().naive_local();

    // parse mentions such as @foo @bar
    let re = Regex::new(r"\B@([\da-zA-Z_]+)").unwrap();
    let config = req.get::<Read<Config>>().unwrap();
    let app_path = config.get("app_path").as_str().unwrap();

    let new_content = re.replace_all(&content, |cap: &Captures| {
        match handle_mention(&pool, cap.at(1).unwrap()) {
            Some(user_id) => format!("[@{}]({}{}{})", cap.at(1).unwrap(), app_path, "/user/", user_id),
            None => format!("@{}", cap.at(1).unwrap()),
        }
    });

    let mut trans = pool.start_transaction(false, None, None).unwrap();
    trans.prep_exec(r"INSERT INTO comment(article_id, user_id, content, create_time) VALUES (?, ?, ?, ?)",
                       (article_id, user.id, new_content, now)).unwrap();
    trans.prep_exec(r"UPDATE article set comments_count=comments_count+1 where id=?", (article_id,)).unwrap();
    trans.commit().unwrap();

    json_ok_response()
}

fn handle_mention(pool: &my::Pool, username: &str) -> Option<u64> {
    // TODO: add some events to notify users

    pool.prep_exec("SELECT id from user where username=?", (username,)).unwrap()
        .next().map(|row| my::from_row(row.unwrap()))
}
