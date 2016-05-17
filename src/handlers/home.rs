use iron::prelude::*;
use base::framework::{ResponseData, temp_response, not_found_response};
use base::db::MyPool;
use persistent::Read;
use base::model::{Article, User, Category};
use mysql as my;
use mysql::QueryResult;
use rustc_serialize::json::{Object, Array, Json, ToJson};
use router::Router;
use base::util::gen_gravatar_url;
use base::constant;

pub fn index(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let result = pool.prep_exec("SELECT a.id, a.category, a.title, a.content, a.comments_count, a.create_time, \
                                 u.id as user_id, u.username, u.email from article \
                                 as a join user as u on a.user_id=u.id", ()).unwrap();

    index_data(req, result, None)
}

pub fn category(req: &mut Request) -> IronResult<Response> {
    let category_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("category_id").unwrap()
                       .parse::<u8>().map_err(|_| not_found_response().unwrap_err()));

    if Category::all().into_iter().find(|c|c.get_value() == category_id).is_none() {
        return not_found_response();
    }

    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let result = pool.prep_exec("SELECT a.id, a.category, a.title, a.content, a.comments_count, a.create_time, \
                                     u.id as user_id, u.username, u.email from article \
                                     as a join user as u on a.user_id=u.id where a.category=?", (category_id,)).unwrap();

    index_data(req, result, Some(category_id))
}

fn index_data(req: &mut Request, result: QueryResult, raw_category_id: Option<u8>) -> IronResult<Response> {
    let articles: Vec<Article> = result.map(|x| x.unwrap()).map(|row| {
        let (id, category, title, content, comments_count, create_time, user_id, username, email) = my::from_row::<(_,_,_,_,_,_,_,_,String)>(row);
        Article {
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
            comments: Vec::new(),
        }
    }).collect();

    let mut data = ResponseData::new(req);
    data.insert("articles", articles.to_json());
    if let Some(category_id) = raw_category_id {
        data.insert("categories", gen_categories_json_with_active_state(category_id));
    } else {
        data.insert("categories", Category::all().to_json());
        data.insert("index", 1.to_json());
    }
    temp_response("index", &data)
}

fn gen_categories_json_with_active_state(active_value: u8) -> Json {
    let mut categories = Array::new();

    for category in &Category::all() {
        let mut object = Object::new();
        object.insert("value".to_owned(), category.get_value().to_json());
        object.insert("title".to_owned(), category.get_title().to_json());
        object.insert("active".to_owned(), (if category.get_value() == active_value {1} else {0}).to_json());
        categories.push(object.to_json());
    }

    categories.to_json()
}
