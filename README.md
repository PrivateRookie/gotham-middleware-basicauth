# gotham-middleware-basicauth
http basic auth middleware for Gotham framework

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![996.ICU](https://img.shields.io/badge/link-996.icu-red.svg)](https://996.icu)
[![](https://tokei.rs/b1/github/PrivateRookie/gotham-middleware-basicauth)](https://tokei.rs/b1/github/PrivateRookie/gotham-middleware-basicauth)

## Usage

example code is in `examples/basic-auth`, to run example, under repo root dir, just run

```bash
cargo run --example basic-auth
```

and then open `http://localhost:8000` on your browser, fill "admin" as username and password in
login modal, then you will see index page.

## todo

- [ ] Add scoped protecd path
- [ ] More elegant error handle and log output
- [ ] Add doc
- [ ] Add unit test
- [ ] Extend protocol to enable logout and login control
- [ ] publish to crates.io