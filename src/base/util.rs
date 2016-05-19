use pulldown_cmark::Parser;
use pulldown_cmark::html;
use crypto::md5;
use crypto::digest::Digest;
use std::collections::HashSet;
use ammonia::Ammonia;
use rustc_serialize::json::{Object, Json, Array, ToJson};
use base::model::Category;

pub fn render_html(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = Parser::new(&text);
    html::push_html(&mut s, p);
    let mut cleaner = Ammonia::default();
    let mut code_attributes = HashSet::new();
    code_attributes.insert("class");
    cleaner.tag_attributes.insert("code", code_attributes);
    cleaner.clean(&*s).to_owned()
}

pub fn gen_gravatar_url(email: &str) -> String {
    let mut sh = md5::Md5::new();
    sh.input_str(&email.trim().to_lowercase());
    "https://cdn.v2ex.com/gravatar/".to_owned() + &sh.result_str()
}

pub fn gen_categories_json_with_active_state(active_value: u8) -> Json {
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
