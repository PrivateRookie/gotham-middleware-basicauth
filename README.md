# gotham-middleware-basicauth
http basic auth middleware for Gotham framework

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