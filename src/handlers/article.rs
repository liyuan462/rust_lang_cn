use iron::prelude::*;
use base::framework::{ResponseData, temp_response,
                      json_error_response, json_ok_response,
                      not_found_response};
use urlencoded::UrlEncodedBody;
use base::db::MyPool;
use base::validator::{Validator, Checker, Str, StrValue, Int, Max, Min};
use base::framework::LoginUser;
use iron_login::User as U;
use persistent::Read;
use chrono::*;
use router::Router;
use mysql as my;
use base::model::{Article, User};
use rustc_serialize::json::{Object, Json, ToJson, encode};

pub fn new_load(req: &mut Request) -> IronResult<Response> {
    let data = ResponseData::new(req);
    temp_response("article/new_load", &data)
}

pub fn new(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("title", Str, "标题") << Min(3) << Max(48))
        .add_checker(Checker::new("content", Str, "内容") << Min(7));

    validator.validate(req.get::<UrlEncodedBody>());
    if !validator.is_valid() {
        return json_error_response(&validator.messages[0]);
    }

    let title = validator.get_valid::<StrValue>("title").value();
    let content = validator.get_valid::<StrValue>("content").value();
    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let login = LoginUser::get_login(req);
    let user = login.get_user().unwrap();

    let now = Local::now().naive_local();
    let mut stmt = pool.prepare(r"INSERT INTO article(title, content, user_id, create_time) VALUES (?, ?, ?, ?)").unwrap();
    let result = stmt.execute((title, content, user.id, now));
    result.unwrap();
    json_ok_response()
}

pub fn show(req: &mut Request) -> IronResult<Response> {
    let mut article_id = 0;
    {
        let raw_article_id = req.extensions.get::<Router>().unwrap().find("article_id").unwrap();
        let wrapped_article_id = raw_article_id.parse::<u64>();
        if wrapped_article_id.is_err() {
            return not_found_response();
        }
        article_id = wrapped_article_id.unwrap();
    }
    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut result = pool.prep_exec("SELECT a.id, a.title, a.content, a.create_time, \
                                     u.id as user_id, u.username, u.email from article \
                                     as a join user as u on a.user_id=u.id where a.id=?", (&article_id,)).unwrap();

    let raw_row = result.next();
    if raw_row.is_none() {
        return not_found_response();
    }
    let row = raw_row.unwrap().unwrap();
    let (id, title, content, create_time, user_id, username, email) = my::from_row(row);
    let article = Article {
        id: id,
        title: title,
        content: content,
        user: User{
            id: user_id,
            username: username,
            email: email,
        },
        create_time: create_time,
    };
    let mut data = ResponseData::new(req);
    data.insert("article".to_owned(), article.to_json());
    temp_response("article/show", &data)
}
