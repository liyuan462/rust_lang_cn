use chrono::*;
use rustc_serialize::json::{Object, Json, ToJson, encode};

pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
}

impl ToJson for User {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("username".to_owned(), self.username.to_json());
        object.insert("email".to_owned(), self.email.to_json());
        object.to_json()
    }
}

pub struct Article {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub user: User,
    pub create_time: NaiveDateTime,
}

impl ToJson for Article {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("title".to_owned(), self.title.to_json());
        object.insert("content".to_owned(), self.content.to_json());
        object.insert("user".to_owned(), self.user.to_json());
        object.insert("create_time".to_owned(), self.create_time.format("%Y-%m-%d %H:%M:%S").to_string().to_json());
        object.to_json()
    }
}
