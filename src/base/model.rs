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

impl Default for User {
    fn default() -> User {
        User {
            id: Default::default(),
            username: Default::default(),
            email: Default::default(),
            avatar: Default::default(),
            create_time: *constant::DEFAULT_DATETIME,
        }
    }
}

impl ToJson for User {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("username".to_owned(), self.username.to_json());
        object.insert("email".to_owned(), self.email.to_json());
        object.insert("avatar".to_owned(), self.avatar.to_json());
        object.insert("create_time".to_owned(),
                      self.create_time.format("%Y-%m-%d %H:%M:%S").to_string().to_json());
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
    pub update_time: NaiveDateTime,
    pub comments: Vec<Comment>,
    pub flag: u8,
}

impl Default for Article {
    fn default() -> Article {
        Article {
            id: Default::default(),
            category: Default::default(),
            title: Default::default(),
            content: Default::default(),
            user: Default::default(),
            comments_count: Default::default(),
            create_time: *constant::DEFAULT_DATETIME,
            update_time: *constant::DEFAULT_DATETIME,
            comments: Default::default(),
            flag: Default::default(),
        }
    }
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
        object.insert("create_time".to_owned(),
                      self.create_time
                          .format("%Y-%m-%d %H:%M:%S")
                          .to_string()
                          .to_json());
        object.insert("update_time".to_owned(),
                      self.update_time
                          .format("%Y-%m-%d %H:%M:%S")
                          .to_string()
                          .to_json());
        object.insert("is_top".to_owned(),
                      (self.flag & constant::ARTICLE::FLAG::TOP > 0).to_json());
        object.insert("is_essence".to_owned(),
                      (self.flag & constant::ARTICLE::FLAG::ESSENCE > 0).to_json());
        object.insert("comments".to_owned(), self.comments.to_json());
        object.to_json()
    }
}

pub struct Comment {
    pub id: u64,
    pub user: User,
    pub content: String,
    pub create_time: NaiveDateTime,
    pub article: Option<Article>,
}

impl ToJson for Comment {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("content".to_owned(), self.content.to_json());
        object.insert("user".to_owned(), self.user.to_json());
        object.insert("create_time".to_owned(),
                      self.create_time.format("%Y-%m-%d %H:%M:%S").to_string().to_json());
        object.insert("article".to_owned(), self.article.to_json());
        object.to_json()
    }
}

pub struct Category {
    pub value: i8,
    pub title: String,
}

impl Default for Category {
    fn default() -> Category {
        Category::from_value(0)
    }
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
            title: (*constant::CATEGORY::TITLES.get(&value).unwrap()).to_owned(),
        }
    }
}
