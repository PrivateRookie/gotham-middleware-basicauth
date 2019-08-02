use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use gotham_middleware_basicauth::AuthMiddleWare;

fn router() -> Router {
    // allow user admin login with password "admin"
    let userlist = vec!["admin:admin".to_owned()];
    let (chain, pipeline) = single_pipeline(
        new_pipeline().add(AuthMiddleWare::new(userlist)).build(),
    );

    build_router(chain, pipeline, |route| {
        route.get("/").to(say_hello);
    })
}

fn say_hello(state: State) -> (State, &'static str) {
    (state, "Hello Auththorized User!")
}

fn main() {
    let addr = "0.0.0.0:8000";
    println!("gotham is running on 0.0.0.0:8000, login with admin/admin");
    gotham::start(addr, router());
}