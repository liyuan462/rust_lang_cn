use chrono::*;
use rustc_serialize::json::{Object, Json, ToJson};

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
    pub category: Category,
    pub title: String,
    pub content: String,
    pub user: User,
    pub create_time: NaiveDateTime,
}

impl ToJson for Article {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("id".to_owned(), self.id.to_json());
        object.insert("category".to_owned(), self.category.to_json());
        object.insert("title".to_owned(), self.title.to_json());
        object.insert("content".to_owned(), self.content.to_json());
        object.insert("user".to_owned(), self.user.to_json());
        object.insert("create_time".to_owned(), self.create_time.format("%Y-%m-%d %H:%M:%S").to_string().to_json());
        object.to_json()
    }
}

// #[derive(Clone)]
// pub struct Category {
//     pub value: i8,
//     pub title: String,
// }

// impl ToJson for Category {
//     fn to_json(&self) -> Json {
//         let mut object = Object::new();
//         object.insert("value".to_owned(), self.value.to_json());
//         object.insert("title".to_owned(), self.title.to_json());
//         object.to_json()
//     }
// }

// impl<'a> ToJson for &'a Category {
//     fn to_json(&self) -> Json {
//         self.clone().to_json()
//     }
// }

// impl Category {
//     pub fn new(value: i8, title: &str) -> Category {
//         Category {
//             value: value,
//             title: title.to_owned(),
//         }
//     }

//     pub fn from(value: u8) -> Category {
//         match value {

//         }
//     }
// }

pub enum Category {
    Resource,
    Question,
    Recruit,
}

impl Category {
    pub fn get_value(&self) -> u8 {
        match *self {
            Category::Resource => 0,
            Category::Question => 1,
            Category::Recruit => 2,
        }
    }

    pub fn get_title(&self) -> String {
        match *self {
            Category::Resource => "学习资源".to_owned(),
            Category::Question => "问答".to_owned(),
            Category::Recruit => "招聘".to_owned(),
        }
    }

    pub fn from_value(value: u8) -> Category {
        match value {
            0 => Category::Resource,
            1 => Category::Question,
            2 => Category::Recruit,
            _ => unreachable!(),
        }
    }

    pub fn all() -> Vec<Category> {
        vec![Category::Resource,
         Category::Question,
         Category::Recruit]
    }
}

impl ToJson for Category {
    fn to_json(&self) -> Json {
        let mut object = Object::new();
        object.insert("value".to_owned(), self.get_value().to_json());
        object.insert("title".to_owned(), self.get_title().to_json());
        object.to_json()
    }
}
