use iron::prelude::*;
use base::framework::{ResponseData, temp_response, not_found_response};
use base::db::MyPool;
use persistent::Read;
use base::model::{Article, User, Category};
use mysql as my;
use mysql::QueryResult;
use rustc_serialize::json::{Object, Json, Array, ToJson};
use router::Router;
use base::util::gen_gravatar_url;
use base::constant;
use base::util;
use urlencoded::UrlEncodedQuery;
use base::validator::{Validator, Checker, Int, IntValue, Min, Optional};
use base::framework::LoginUser;
use iron_login::User as U;
use iron::status;
use rss::{Channel, Item, Rss, Guid};
use chrono::*;
use base::config::Config;
use base::util::render_html;

pub fn index(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator.add_checker(Checker::new("page", Int, "页码") << Min(1) << Optional);
    validator.validate(req.get::<UrlEncodedQuery>());
    if !validator.is_valid() {
        return not_found_response();
    }

    let page = match validator.valid_data.get("page") {
        Some(p) => p[0].downcast_ref_unchecked::<IntValue>().value(),
        None => 1,
    } as usize;

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let row = pool.prep_exec("SELECT count(id) from article where status=? ",
                             (constant::ARTICLE_STATUS::NORMAL,))
        .unwrap().next().unwrap().unwrap();

    let count: usize = my::from_row(row);
    let page_count = (count + constant::PAGE_SIZE - 1) / constant::PAGE_SIZE;

    let result = pool.prep_exec(
        "SELECT a.id, a.category, a.title, a.content, a.comments_count, \
         a.create_time, u.id as user_id, u.username, u.email from article \
         as a join user as u on a.user_id=u.id where a.status=? \
         order by a.priority desc, a.create_time desc limit ?,?",
        (constant::ARTICLE_STATUS::NORMAL,
         (page - 1) * constant::PAGE_SIZE,
         constant::PAGE_SIZE)).unwrap();

    index_data(req, &pool, page, page_count, result, None)
}

pub fn category(req: &mut Request) -> IronResult<Response> {
    let category_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("category_id").unwrap()
                       .parse::<i8>().map_err(|_| not_found_response().unwrap_err()));

    if constant::CATEGORY::ALL.iter().find(|c|**c == category_id).is_none() {
        return not_found_response();
    }

    let mut validator = Validator::new();
    validator.add_checker(Checker::new("page", Int, "页码") << Min(1) << Optional);
    validator.validate(req.get::<UrlEncodedQuery>());
    if !validator.is_valid() {
        return not_found_response();
    }

    let page = match validator.valid_data.get("page") {
        Some(p) => p[0].downcast_ref_unchecked::<IntValue>().value(),
        None => 1,
    } as usize;

    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let row = pool.prep_exec("SELECT count(id) from article where status=? \
                              and category=?",
                             (constant::ARTICLE_STATUS::NORMAL, category_id))
        .unwrap().next().unwrap().unwrap();
    let count: usize = my::from_row(row);
    let page_count = (count + constant::PAGE_SIZE - 1) / constant::PAGE_SIZE;

    let result = pool.prep_exec(
        "SELECT a.id, a.category, a.title, a.content, a.comments_count, \
         a.create_time, u.id as user_id, u.username, u.email from article \
         as a join user as u on a.user_id=u.id where a.status=? and a.category=? \
         order by a.priority desc, a.create_time desc limit ?,?",
        (constant::ARTICLE_STATUS::NORMAL,
         category_id,
         (page - 1) * constant::PAGE_SIZE,
         constant::PAGE_SIZE)).unwrap();

    index_data(req, &pool, page, page_count, result, Some(category_id))
}

fn index_data(
    req: &mut Request, pool: &my::Pool,
    page: usize, page_count: usize,
    result: QueryResult, raw_category_id: Option<i8>) -> IronResult<Response> {

    let articles: Vec<Article> = result.map(|x| x.unwrap()).map(|row| {
        let (id, category, title, content, comments_count,
             create_time, user_id, username, email) = my::from_row::<
                (_,_,_,_,_,_,_,_,String)>(row);

        Article {
            id: id,
            category: Category::from_value(category),
            title: title,
            content: content,
            comments_count: comments_count,
            user: User {
                id: user_id,
                avatar: gen_gravatar_url(&email),
                username: username,
                email: email,
                create_time: *constant::DEFAULT_DATETIME,
            },
            create_time: create_time,
            comments: Vec::new(),
        }
    }).collect();

    // get statistics info
    let users_count = my::from_row::<usize>(
        pool.prep_exec("SELECT count(id) as count from user", ())
            .unwrap().next().unwrap().unwrap());
    let articles_count = my::from_row::<usize>(
        pool.prep_exec("SELECT count(id) as count from article", ())
            .unwrap().next().unwrap().unwrap());

    let mut data = ResponseData::new(req);
    let show_pagination = if page_count > 1 {true} else {false};
    data.insert("show_pagination", show_pagination.to_json());
    data.insert("pages", gen_pages_json(page_count, page));
    data.insert("previous_page",
                (if page - 1 < 1 {1} else {page - 1}).to_json());
    data.insert("next_page",
                (if page + 1 > page_count {page_count} else {page + 1}).to_json());

    data.insert("articles", articles.to_json());
    data.insert("users_count", users_count.to_json());
    data.insert("articles_count", articles_count.to_json());

    if let Some(category_id) = raw_category_id {
        data.insert("categories", util::gen_categories_json(Some(category_id)));
        data.insert("category", category_id.to_json());
    } else {
        data.insert("categories", util::gen_categories_json(None));
        data.insert("index", 1.to_json());
    }

    // get unread messages
    let mut unread_messages_count:usize = 0;
    let raw_login_user = LoginUser::get_login(req).get_user();
    if let Some(login_user) = raw_login_user {
        unread_messages_count = my::from_row(pool.prep_exec(
            "SELECT count(id) as count from message where to_user_id=? and status=?",
            (login_user.id, constant::MESSAGE::STATUS::INIT))
            .unwrap().next().unwrap().unwrap());
    }
    data.insert("unread_messages_count", unread_messages_count.to_json());
    temp_response("index", &data)
}

pub fn rss(req: &mut Request) -> IronResult<Response> {
    let config = req.get::<Read<Config>>().unwrap();
    let app_path = config.get("app_path").as_str().unwrap().to_owned();

    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let now = Local::now();
    let offset = now.offset().clone();

    let result = pool.prep_exec(
        "SELECT a.id, a.title, a.content,
         a.create_time, u.username from article \
         as a join user as u on a.user_id=u.id where a.status=? \
         order by a.create_time desc limit ?,?",
        (constant::ARTICLE_STATUS::NORMAL,
         0,
         constant::PAGE_SIZE)).unwrap();

    let items: Vec<Item> = result.map(|x| x.unwrap()).map(|row| {
        let (id, title, content, create_time, username) = my::from_row::<(
            u64, String, String, NaiveDateTime, String)>(row);

        let article_path = format!("{}/article/{}", app_path, id);

        let create_time_with_tz = DateTime::<Local>::from_utc(
            create_time - offset.local_minus_utc(), offset);

        Item {
            title: Some(util::safe_xml(&title)),
            pub_date: Some(create_time_with_tz.to_rfc2822()),
            description: Some(util::safe_xml(&render_html(&content))),
            author: Some(username),
            link: Some(article_path.clone()),
            guid: Some(Guid{is_perma_link: true, value: article_path}),
            ..Default::default()
        }
    }).collect();

    let channel = Channel {
        title: String::from("Rust China社区"),
        link: app_path,
        description: String::from("Rust China社区最新发帖"),
        language: Some(String::from("zh-cn")),
        items: items,
        ..Default::default()
    };

    let rss = Rss(channel);
    let rss_string = rss.to_string();

    let mut resp = Response::new();
    resp.set_mut(mime!(Application/Xml))
        .set_mut(rss_string)
        .set_mut(status::Ok);
    Ok(resp)
}

fn gen_pages_json(page_count: usize, current_page: usize) -> Json {
    let mut pages = Array::new();

    for page in 1..page_count + 1 {
        let mut object = Object::new();
        object.insert("page".to_owned(), page.to_json());
        if page == current_page {
            object.insert("active".to_owned(), 1.to_json());
        }
        pages.push(object.to_json());
    }

    pages.to_json()
}
