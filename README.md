# gotham-middleware-basicauth
http basic auth middleware for Gotham framework

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![996.ICU](https://img.shields.io/badge/link-996.icu-red.svg)](https://996.icu)
[![Build Status](https://travis-ci.com/PrivateRookie/gotham-middleware-basicauth.svg?branch=master)](https://travis-ci.com/PrivateRookie/gotham-middleware-basicauth)
[![](https://tokei.rs/b1/github/PrivateRookie/gotham-middleware-basicauth)](https://tokei.rs/b1/github/PrivateRookie/gotham-middleware-basicauth)

## Usage

this code take from `examples/basic-auth/main.rs`
```rust
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use gotham_middleware_basicauth::AuthMiddleware;

fn router() -> Router {
    // default allow user admin login with password "admin"
    // and protect all paths
    let (chain, pipeline) = single_pipeline(
        new_pipeline().add(AuthMiddleware::default()).build(),
    );

    build_router(chain, pipeline, |route| {
        route.get("/").to(say_hello);
        route.get("/abc").to(say_hello);
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
```

as you can ses, it's eazy to use, `default` method return a middleware that required
to login as admin with password admin when visiting web site at the first time.

You can create a new middleware manually, codes in example `scoped-auth` show how to create a new middleware:

```rust
let middleware: AuthMiddleware = AuthMiddleware {
    userlist: vec!["admin:admin".to_owned()],
    scopes: vec!["/scoped".to_owned()],
    };
```

You can pass a list of user with format "username:password", and a list of path you want to
protect by basic auth.
Note that if a path is protected, it's subpath will be protected too.

To run these examples, run

```bash
cargo run --example basic-auth
```

or

```bash
cargo run --example scoped-auth
```

and then open `http://localhost:8000` on your browser.

## todo

- [x] ~~Add scoped protecd path~~
- [x] ~~More elegant error handle and log output~~
- [x] ~~Add doc~~
- [x] ~~Add unit test~~
- [ ] Extend protocol to enable logout and login control
- [x] ~~publish to crates.io~~
- [x] ~~doc for basic use and scoped path feature~~