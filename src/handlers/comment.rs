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
use base::constant;

pub fn new(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("article_id", Int, "文章ID") << Min(1))
        .add_checker(Checker::new("content", Str, "内容") << Min(7));

    validator.validate(req.get::<UrlEncodedBody>());
    if !validator.is_valid() {
        return json_error_response(&validator.messages[0]);
    }

    let article_id = validator.get_valid::<IntValue>("article_id").value() as u64;

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut trans = pool.start_transaction(false, None, None).unwrap();

    // check whether article exists
    let raw_row = trans.prep_exec("SELECT user_id from article where id=? for update",
                                  (&article_id,)).unwrap().next();
    if raw_row.is_none() {
            return not_found_response();
    };

    let article_user_id: u64 = my::from_row(raw_row.unwrap().unwrap());

    let content = validator.get_valid::<StrValue>("content").value();
    let login = LoginUser::get_login(req);
    let user = login.get_user().unwrap();
    let now = Local::now().naive_local();

    // parse mentions such as @foo @bar
    let re = Regex::new(r"\B@([\da-zA-Z_]+)").unwrap();
    let config = req.get::<Read<Config>>().unwrap();
    let app_path = config.get("app_path").as_str().unwrap();

    let mut mentions: Vec<u64> = Vec::new();
    let new_content = re.replace_all(&content, |cap: &Captures| {
        match handle_mention(&mut trans, cap.at(1).unwrap()) {
            Some(user_id) => {
                mentions.push(user_id);
                format!("[@{}]({}{}{})",
                        cap.at(1).unwrap(),
                        app_path,
                        "/user/",
                        user_id)
            },
            None => format!("@{}", cap.at(1).unwrap()),
        }
    });

    let comment_id = trans.prep_exec(
        "INSERT INTO comment(article_id, user_id, content, create_time) \
         VALUES (?, ?, ?, ?)",
        (article_id, user.id, new_content, now)).unwrap().last_insert_id();

    trans.prep_exec("UPDATE article set comments_count=comments_count+1, \
                     update_time=? where id=?",
                    (article_id, now)).unwrap();

    // send message to article's author
    if article_user_id != user.id {
        trans.prep_exec("INSERT INTO message(article_id, comment_id, \
                         from_user_id, to_user_id, mode, \
                         status, create_time) VALUES (?, ?, ?, ?, ?, ?, ?)",
                        (article_id, comment_id, user.id, article_user_id,
                         constant::MESSAGE::MODE::REPLY_ARTICLE,
                         constant::MESSAGE::STATUS::INIT, now)).unwrap();
    }

    // send message to mentions
    mentions.sort();
    mentions.dedup();
    for mention in mentions.iter().filter(|&x| *x != article_user_id && *x != user.id) {
        trans.prep_exec("INSERT INTO message(article_id, comment_id, \
                         from_user_id, to_user_id, mode, \
                         status, create_time) VALUES (?, ?, ?, ?, ?, ?, ?)",
                        (article_id, comment_id, user.id, mention,
                         constant::MESSAGE::MODE::MENTION,
                         constant::MESSAGE::STATUS::INIT, now)).unwrap();
    }

    trans.commit().unwrap();

    json_ok_response()
}

fn handle_mention(trans: &mut my::Transaction, username: &str) -> Option<u64> {

    trans.prep_exec("SELECT id from user where username=?", (username,)).unwrap()
        .next().map(|row| my::from_row(row.unwrap()))
}
