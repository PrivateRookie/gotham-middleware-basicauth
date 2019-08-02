use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham_middleware_basicauth::AuthMiddleWare;

fn router() -> Router {
    // allow user admin login with passwod "admin"
    let userlist = vec!["admin:admin".to_owned()];
    let (chain, pipeline) = single_pipeline(
        new_pipeline().add(AuthMiddleWare::new(userlist)).build(),
    );

    build_router(chain, pipeline, |route| {
        route.get("/").to_file("src/static/index.html");
    })
}

fn main() {
    let addr = "0.0.0.0:8000";
    gotham::start(addr, router());
}