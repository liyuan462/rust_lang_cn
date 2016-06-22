use chrono::*;

lazy_static! {
    pub static ref DEFAULT_DATETIME: NaiveDateTime = UTC.ymd(
        1970,1,1).and_hms(0, 0, 0).naive_local();
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod ARTICLE {
    pub mod STATUS {
        pub const NORMAL: i8 = 0;
        pub const DELETED: i8 = -1;
    }

    pub mod FLAG {
        pub const TOP: u8 = 1 << 0;
        pub const ESSENCE: u8 = 1 << 1;
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod CATEGORY {
    use std::collections::HashMap;

    pub const COMMUNITY: i8 = 0;
    pub const ARTICLES: i8 = 1;
    pub const EVERYDAY: i8 = 2;
    pub const WEEKLY: i8 = 3;
    pub const DOC: i8 = 4;
    pub const RESOURCES: i8 = 5;
    pub const JOBS: i8 = 6;
    pub const IRC: i8 = 7;

    lazy_static! {
        pub static ref ALL: Vec<i8> = vec![
            COMMUNITY, ARTICLES, EVERYDAY, WEEKLY, DOC, RESOURCES, JOBS, IRC];

        pub static ref TITLES: HashMap<i8, &'static str> = m_hashmap![
            COMMUNITY => "社区",           //向 reddit + Stack Overflow 形式演化
            ARTICLES => "文章",            //同golang中国和go语言中文网的文章板块
            EVERYDAY => "EveryDay",       //每日两个rust代码段例子，之一同步Rust官方twitter的ervery rust帐号例子，之二由社区贡献。
            WEEKLY => "Weekly",           //同步官方this week in rust + 国内 this week in rust
            DOC => "文档",                //重要文档中文化。
            RESOURCES => "资源",          //包括：开源项目，名库，图书，视频，名站... 利于国内项目学习和发展。
            JOBS => "招聘",               //关于招聘信息,
            IRC => "IRC"                 //网络在线即时聊天，交流。
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
