use router::Router;
use handlers;

pub fn gen_router() -> Router {
    let mut router = Router::new();
    router.get("/", handlers::home::index);
    router
}
