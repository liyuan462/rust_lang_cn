use router::Router;
use handlers;

pub fn gen_router() -> Router {
    let mut router = Router::new();
    router.get("/", handlers::home::index);
    router.get("/user/register/load", handlers::user::register_load);
    router.post("/user/register", handlers::user::register);
    router
}
