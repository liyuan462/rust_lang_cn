use chrono::*;

lazy_static! {
    pub static ref DEFAULT_DATETIME: NaiveDateTime = UTC.ymd(1970,1,1).and_hms(0, 0, 0).naive_local();
}
