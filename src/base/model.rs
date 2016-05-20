use chrono::*;
use rustc_serialize::json::{Object, Json, ToJson};
use base::constant;

pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub create_time: NaiveDateTime,
}

impl ToJson for User {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("username".to_owned(), self.username.to_json());
        object.insert("email".to_owned(), self.email.to_json());
        object.insert("avatar".to_owned(), self.avatar.to_json());
        object.insert("create_time".to_owned(), self.create_time.format("%Y-%m-%d %H:%M:%S").to_string().to_json());
        object.to_json()
    }
}

pub struct Article {
    pub id: u64,
    pub category: Category,
    pub title: String,
    pub content: String,
    pub user: User,
    pub comments_count: usize,
    pub create_time: NaiveDateTime,
    pub comments: Vec<Comment>,
}

impl ToJson for Article {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("category".to_owned(), self.category.to_json());
        object.insert("title".to_owned(), self.title.to_json());
        object.insert("content".to_owned(), self.content.to_json());
        object.insert("user".to_owned(), self.user.to_json());
        object.insert("comments_count".to_owned(), self.comments_count.to_json());
        object.insert("create_time".to_owned(), self.create_time.format("%Y-%m-%d %H:%M:%S").to_string().to_json());
        object.insert("comments".to_owned(), self.comments.to_json());
        object.to_json()
    }
}

pub struct Comment {
    pub id: u64,
    pub user: User,
    pub content: String,
    pub create_time: NaiveDateTime,
}

impl ToJson for Comment {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("content".to_owned(), self.content.to_json());
        object.insert("user".to_owned(), self.user.to_json());
        object.insert("create_time".to_owned(), self.create_time.format("%Y-%m-%d %H:%M:%S").to_string().to_json());
        object.to_json()
    }
}

pub struct Category {
    pub value: i8,
    pub title: String,
}

impl ToJson for Category {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("value".to_owned(), self.value.to_json());
        object.insert("title".to_owned(), self.title.to_json());
        object.to_json()
    }
}

impl Category {
    pub fn from_value(value: i8) -> Category {
        Category {
            value: value,
            title: (*constant::CATEGORY::TITLES.get(&value).unwrap()).to_owned()
        }
    }
}
