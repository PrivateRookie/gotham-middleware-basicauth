# gotham-middleware-basicauth
http basic auth middleware for Gotham framework

## Usage

```rust
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham_cors_middleware::CORSMiddleware;
use gotham::router::builder::*;
use gotham::router::Router;

fn router() -> Router {
    let (chain, pipeline) = single_pipeline
}
```

## todo

- [ ] Add scoped protecd path
- [ ] More elegant error handle and log output
- [ ] Add doc
- [ ] Add unit test