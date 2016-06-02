use chrono::*;

lazy_static! {
    pub static ref DEFAULT_DATETIME: NaiveDateTime = UTC.ymd(1970,1,1).and_hms(0, 0, 0).naive_local();
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod ARTICLE_STATUS {
    pub const NORMAL: i8 = 0;
    pub const DELETED: i8 = -1;
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod CATEGORY {
    use std::collections::HashMap;

    pub const NONSENSE: i8 = 0;
    pub const ORIGINAL: i8 = 1;
    pub const FORWARD: i8 = 2;
    pub const TRANSLATION: i8 = 3;
    pub const QUESTION: i8 = 4;
    pub const RECRUIT: i8 = 5;
    pub const SITE: i8 = 6;

    lazy_static! {
        pub static ref ALL: Vec<i8> = collect![ORIGINAL, FORWARD, TRANSLATION, QUESTION, RECRUIT, NONSENSE, SITE];

        pub static ref TITLES: HashMap<i8, &'static str> = collect![
            NONSENSE => "扯淡",
            ORIGINAL => "原创",
            FORWARD => "转载",
            TRANSLATION => "翻译",
            QUESTION => "提问",
            RECRUIT => "招聘",
            SITE => "站务"
        ];
    }

}

pub const PAGE_SIZE: usize = 15;

#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod MESSAGE {
    pub mod MODE {
        pub const REPLY_ARTICLE: i8 = 1;       // 文章下面回复
        pub const MENTION: i8 = 2;             // 在回复中提到某人
    }

    pub mod STATUS {
        pub const INIT: i8 = 0;                // 初始
        pub const READ: i8 = 1;                // 已读
    }
}
