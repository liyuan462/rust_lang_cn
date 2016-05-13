use iron::prelude::*;
use base::framework::{ResponseData, temp_response, not_found_response};
use base::db::MyPool;
use persistent::Read;
use base::model::{Article, User, Category};
use mysql as my;
use rustc_serialize::json::{Object, Array, Json, ToJson};
use router::Router;

pub fn index(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let result = pool.prep_exec("SELECT a.id, a.category, a.title, a.content, a.create_time, \
                                     u.id as user_id, u.username, u.email from article \
                                     as a join user as u on a.user_id=u.id", ()).unwrap();
    let articles: Vec<Article> = result.map(|x| x.unwrap()).map(|row| {
        let (id, category, title, content, create_time, user_id, username, email) = my::from_row(row);
        Article {
            id: id,
            category: Category::from_value(category),
            title: title,
            content: content,
            user: User{
                id: user_id,
                username: username,
                email: email,
            },
            create_time: create_time,
        }
    }).collect();
    let mut data = ResponseData::new(req);
    data.insert("articles".to_owned(), articles.to_json());
    data.insert("categories".to_owned(), Category::all().to_json());
    data.insert("index".to_owned(), 1.to_json());
    temp_response("index", &data)
}

pub fn category(req: &mut Request) -> IronResult<Response> {
    let category_id;
    {
        let raw_category_id = req.extensions.get::<Router>().unwrap().find("category_id").unwrap();
        let wrapped_category_id = raw_category_id.parse::<u8>();
        if wrapped_category_id.is_err() {
            return not_found_response();
        }
        category_id = wrapped_category_id.unwrap();
    }
    let pool = req.get::<Read<MyPool>>().unwrap().value();

    let result = pool.prep_exec("SELECT a.id, a.category, a.title, a.content, a.create_time, \
                                     u.id as user_id, u.username, u.email from article \
                                     as a join user as u on a.user_id=u.id where a.category=?", (category_id,)).unwrap();

    let articles: Vec<Article> = result.map(|x| x.unwrap()).map(|row| {
        let (id, category, title, content, create_time, user_id, username, email) = my::from_row(row);
        Article {
            id: id,
            category: Category::from_value(category),
            title: title,
            content: content,
            user: User{
                id: user_id,
                username: username,
                email: email,
            },
            create_time: create_time,
        }
    }).collect();
    let mut data = ResponseData::new(req);
    data.insert("articles".to_owned(), articles.to_json());
    data.insert("categories".to_owned(), gen_categories_json_with_active_state(category_id));
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
