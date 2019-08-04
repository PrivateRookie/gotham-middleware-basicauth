use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham_middleware_basicauth::AuthMiddleware;

fn router() -> Router {
    // only request to visit /scope and it's sub path need auth
    let middleware: AuthMiddleware = AuthMiddleware {
        userlist: vec!["admin:admin".to_owned()],
        scopes: vec!["/scoped".to_owned()],
    };
    let (chain, pipeline) = single_pipeline(new_pipeline().add(middleware).build());

    build_router(chain, pipeline, |route| {
        route.get("/").to(|state| (state, "Public Page"));
        route.get("/page").to(|state| (state, "Public Page"));
        route.get("/scoped").to(|state| (state, "Private Page"));
        route
            .get("/scoped/page")
            .to(|state| (state, "Private Page"));
    })
}

fn main() {
    let addr = "0.0.0.0:8000";
    println!("gotham is running on 0.0.0.0:8000, login with admin/admin");
    gotham::start(addr, router());
}
