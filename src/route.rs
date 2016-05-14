use router::Router;
use handlers;
use base::framework::user_required;

pub fn gen_router() -> Router {
    let mut router = Router::new();
    router.get("/", handlers::home::index);
    router.get("/user/register", handlers::user::register_load);
    router.post("/user/register", handlers::user::register);
    router.get("/user/login", handlers::user::login_load);
    router.post("/user/login", handlers::user::login);
    router.post("/user/logout", handlers::user::logout);
    router.get("/article/new", user_required(handlers::article::new_load));
    router.post("/article/new", user_required(handlers::article::new));
    router.get("/article/:article_id", handlers::article::show);
    router.get("/category/:category_id", handlers::home::category);
    router.get("/user/:user_id", handlers::user::show);
    router
}
