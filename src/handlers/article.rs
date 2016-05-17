use iron::prelude::*;
use base::framework::{ResponseData, temp_response,
                      json_error_response, json_ok_response,
                      not_found_response};
use urlencoded::UrlEncodedBody;
use base::db::MyPool;
use base::validator::{Validator, Checker, Str, StrValue, Int, IntValue, Max, Min};
use base::framework::LoginUser;
use base::util::render_html;
use iron_login::User as U;
use persistent::Read;
use chrono::*;
use router::Router;
use mysql as my;
use base::model::{Article, User, Category, Comment};
use rustc_serialize::json::ToJson;
use base::util::gen_gravatar_url;
use base::constant;

pub fn new_load(req: &mut Request) -> IronResult<Response> {
    let mut data = ResponseData::new(req);
    data.insert("categories", Category::all().to_json());
    temp_response("article/new_load", &data)
}

pub fn new(req: &mut Request) -> IronResult<Response> {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("category", Int, "类别") << Min(0) << Max(2))
        .add_checker(Checker::new("title", Str, "标题") << Min(3) << Max(64))
        .add_checker(Checker::new("content", Str, "内容") << Min(7));

    validator.validate(req.get::<UrlEncodedBody>());
    if !validator.is_valid() {
        return json_error_response(&validator.messages[0]);
    }

    let category = validator.get_valid::<IntValue>("category").value();
    let title = validator.get_valid::<StrValue>("title").value();
    let content = validator.get_valid::<StrValue>("content").value();
    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let login = LoginUser::get_login(req);
    let user = login.get_user().unwrap();

    let now = Local::now().naive_local();
    let mut stmt = pool.prepare(r"INSERT INTO article(category, title, content, user_id, create_time) VALUES (?, ?, ?, ?, ?)").unwrap();
    let result = stmt.execute((category, title, content, user.id, now));
    result.unwrap();
    json_ok_response()
}

pub fn show(req: &mut Request) -> IronResult<Response> {
    let article_id = try!(req.extensions.get::<Router>().unwrap()
                       .find("article_id").unwrap()
                       .parse::<u64>().map_err(|_| not_found_response().unwrap_err()));

    let pool = req.get::<Read<MyPool>>().unwrap().value();
    let mut result = pool.prep_exec("SELECT a.id, a.category, a.title, a.content, a.comments_count, a.create_time, \
                                     u.id as user_id, u.username, u.email from article \
                                     as a join user as u on a.user_id=u.id where a.id=?", (&article_id,)).unwrap();

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
        comments: Vec::new(),
    };
    article.content = render_html(&article.content);

    let result = pool.prep_exec("SELECT c.id, c.content, c.create_time, u.id as user_id, u.username, u.email from comment \
                                     as c join user as u on c.user_id=u.id where c.article_id=?", (&article_id, )).unwrap();

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
        }
    }).collect();

    let mut data = ResponseData::new(req);
    data.insert("article", article.to_json());
    let mentions: Vec<String> = article.comments.into_iter().map(|c|c.user.username).collect();
    data.insert("mentions", mentions.to_json());
    temp_response("article/show", &data)
}
