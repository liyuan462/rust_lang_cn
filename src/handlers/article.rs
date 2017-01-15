use std::collections::HashMap;
use iron::prelude::*;
use base::framework::{ResponseData, temp_response,
                      json_error_response, json_ok_response,
                      not_found_response};
use urlencoded::UrlEncodedBody;
use base::db::MyPool;
use form_checker::{Validator, Checker, Rule, Str, I64};
use base::framework::LoginUser;
use base::util::render_html;
use iron_login::User as U;
use persistent::Read;
use chrono::*;
use router::Router;
use mysql as my;
use base::model::{Article, User, Category, Comment};
use rustc_serialize::json::ToJson;
use base::util;
use base::util::gen_gravatar_url;
use base::constant;

pub fn new_load(req: &mut Request) -> IronResult<Response> {
    let mut data = ResponseData::new(req);
    data.insert("categories", util::gen_categories_json(None));
    temp_response("article/new_load", &data)
}

pub fn new(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator
        .check(Checker::new("category", "类别", I64)
               .meet(Rule::Lambda(Box::new(|v| {
                   constant::CATEGORY::ALL.iter().any(
                       |c|*c as i64 == v.as_i64().unwrap())
               }), None)))
        .check(Checker::new("title", "标题", Str)
               .meet(Rule::Min(3))
               .meet(Rule::Max(64)))
        .check(Checker::new("content", "内容", Str)
               .meet(Rule::Min(7)));

    validator.validate(&req.get::<UrlEncodedBody>().unwrap_or(HashMap::new()));
    if !validator.is_valid() {
        return json_error_response(&validator.get_some_error());
    }

    let category = validator.get_required("category").as_i64().unwrap();
    let title = validator.get_required("title").as_str().unwrap();
    let content = validator.get_required("content").as_str().unwrap();
    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let login = LoginUser::get_login(req);
    let user = login.get_user().unwrap();

    let now = Local::now().naive_local();
    let mut stmt = pool.prepare("INSERT INTO article(category, title, content, \
                                 user_id, create_time, update_time) \
                                 VALUES (?, ?, ?, ?, ?, ?)").unwrap();
    let result = stmt.execute((category, title, content, user.id, now, now));
    result.unwrap();
    json_ok_response()
}

pub fn show(req: &mut Request) -> IronResult<Response> {
    let article_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("article_id").unwrap()
                       .parse::<u64>().map_err(|_| not_found_response().unwrap_err()));

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut result = pool.prep_exec(
        "SELECT a.id, a.category, a.title, a.content, a.comments_count, \
         a.create_time, u.id as user_id, u.username, u.email from article \
         as a join user as u on a.user_id=u.id where a.id=? and a.status=?",
        (&article_id, constant::ARTICLE::STATUS::NORMAL)).unwrap();

    let raw_row = result.next();
    if raw_row.is_none() {
        return not_found_response();
    }
    let row = raw_row.unwrap().unwrap();
    let (id, category, title, content, comments_count, create_time, user_id, username, email) = my::from_row::<(_,_,_,_,_,_,_,_,String)>(row);
    let mut article = Article {
        id: id,
        category: Category::from_value(category),
        title: title,
        content: content,
        comments_count: comments_count,
        user: User{
            id: user_id,
            avatar: gen_gravatar_url(&email),
            username: username,
            email: email,
            create_time: *constant::DEFAULT_DATETIME,
        },
        create_time: create_time,
        update_time: *constant::DEFAULT_DATETIME,
        flag: 0,
        comments: Vec::new(),
    };
    article.content = render_html(&article.content);

    let result = pool.prep_exec(
        "SELECT c.id, c.content, c.create_time, u.id as user_id, \
         u.username, u.email from comment \
         as c join user as u on c.user_id=u.id where c.article_id=? \
         order by c.create_time", (&article_id, )).unwrap();

    article.comments = result.map(|x| x.unwrap()).map(|row|{
        let (id, content, create_time, user_id, username, email) = my::from_row::<(_,String,_,_,_,String)>(row);
        Comment {
            id: id,
            content: render_html(&content),
            user: User {
                id: user_id,
                avatar: gen_gravatar_url(&email),
                username: username,
                email: email,
                create_time: *constant::DEFAULT_DATETIME,
            },
            create_time: create_time,
            article: None,
        }
    }).collect();

    // judge whether is my article
    let mut is_my_own = false;
    let raw_login_user = LoginUser::get_login(req).get_user();
    if let Some(login_user) = raw_login_user {
        if login_user.id == article.user.id {
            is_my_own = true;
        }
    }

    let mut data = ResponseData::new(req);
    data.insert("article", article.to_json());
    data.insert("comments_count", article.comments.len().to_json());
    let mentions: Vec<String> = article.comments.into_iter().map(|c|c.user.username).collect();
    data.insert("mentions", mentions.to_json());
    data.insert("is_my_own", is_my_own.to_json());
    temp_response("article/show", &data)
}

pub fn edit_load(req: &mut Request) -> IronResult<Response> {
    let login = LoginUser::get_login(req);
    let user = login.get_user().unwrap();

    let article_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("article_id").unwrap()
                       .parse::<u64>().map_err(|_| not_found_response().unwrap_err()));

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut result = pool.prep_exec(
        "SELECT a.id, a.category, a.title, a.content, a.comments_count, \
         a.create_time, u.id as user_id, u.username, u.email from article \
         as a join user as u on a.user_id=u.id where a.id=? and a.status=?",
        (&article_id, constant::ARTICLE::STATUS::NORMAL)).unwrap();

    let raw_row = result.next();
    if raw_row.is_none() {
        return not_found_response();
    }
    let row = raw_row.unwrap().unwrap();
    let (id, category, title, content, comments_count,
         create_time, user_id, username, email) = my::from_row::<
            (_,_,_,_,_,_,_,_,String)>(row);

    if user_id != user.id {
        return not_found_response();
    }

    let article = Article {
        id: id,
        category: Category::from_value(category),
        title: title,
        content: content,
        comments_count: comments_count,
        user: User{
            id: user_id,
            avatar: gen_gravatar_url(&email),
            username: username,
            email: email,
            create_time: *constant::DEFAULT_DATETIME,
        },
        create_time: create_time,
        update_time: *constant::DEFAULT_DATETIME,
        flag: 0,
        comments: Vec::new(),
    };

    let mut data = ResponseData::new(req);

    data.insert("categories", util::gen_categories_json(Some(category)));
    data.insert("article", article.to_json());
    temp_response("article/edit_load", &data)
}

pub fn edit(req: &mut Request) -> IronResult<Response> {
    let login = LoginUser::get_login(req);
    let user = login.get_user().unwrap();

    let article_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("article_id").unwrap()
                       .parse::<u64>().map_err(|_| not_found_response().unwrap_err()));

    let mut validator = Validator::new();
    validator
        .check(Checker::new("category", "类别", I64)
               .meet(Rule::Lambda(Box::new(|v| {
                   constant::CATEGORY::ALL.iter().any(
                       |c|*c as i64 == v.as_i64().unwrap())
               }), None)))
        .check(Checker::new("title", "标题", Str)
               .meet(Rule::Min(3))
               .meet(Rule::Max(64)))
        .check(Checker::new("content", "内容", Str)
               .meet(Rule::Min(7)));

    validator.validate(&req.get::<UrlEncodedBody>().unwrap_or(HashMap::new()));
    if !validator.is_valid() {
        return json_error_response(&validator.get_some_error());
    }

    let category = validator.get_required("category").as_i64().unwrap();
    let title = validator.get_required("title").as_str().unwrap();
    let content = validator.get_required("content").as_str().unwrap();
    let now = Local::now().naive_local();

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut trans = pool.start_transaction(false, None, None).unwrap();

    {
        let mut result = trans.prep_exec(
            "SELECT u.id as user_id from article \
             as a join user as u on a.user_id=u.id \
             where a.id=? and a.status=? for update",
            (article_id, constant::ARTICLE::STATUS::NORMAL)).unwrap();

        let raw_row = result.next();
        if raw_row.is_none() {
            return not_found_response();
        }

        let row = raw_row.unwrap().unwrap();
        let user_id = my::from_row::<u64>(row);

        if user_id != user.id {
            return json_error_response("非法请求");
        }
    }

    trans.prep_exec("UPDATE article set category=?, title=?, content=?, \
                     update_time=? where id=?",
                    (category, title, content, now, article_id)).unwrap();


    trans.commit().unwrap();

    json_ok_response()
}
