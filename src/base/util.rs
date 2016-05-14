use pulldown_cmark::Parser;
use pulldown_cmark::html;
use crypto::md5;
use crypto::digest::Digest;

pub fn render_html(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = Parser::new(&text);
    html::push_html(&mut s, p);
    s
}

pub fn gen_gravatar_url(email: &str) -> String {
    let mut sh = md5::Md5::new();
    sh.input_str(&email.trim().to_lowercase());
    "https://cdn.v2ex.com/gravatar/".to_owned() + &sh.result_str()
}
