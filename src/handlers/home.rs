use iron::prelude::*;
use base::framework::{ResponseData, temp_response};
use base::framework::LoginUser;
use iron_login::User as U;
use base::db::MyPool;
use persistent::Read;
use base::model::{Article, User};
use mysql as my;
use rustc_serialize::json::{Object, Json, ToJson, encode};

pub fn index(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut result = pool.prep_exec("SELECT a.id, a.title, a.content, a.create_time, \
                                     u.id as user_id, u.username, u.email from article \
                                     as a join user as u on a.user_id=u.id", ()).unwrap();
    let articles: Vec<Article> = result.map(|x| x.unwrap()).map(|row| {
        let (id, title, content, create_time, user_id, username, email) = my::from_row(row);
        Article {
            id: id,
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
    temp_response("index", &data)
}
