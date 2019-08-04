use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::test::TestServer;
use gotham_middleware_basicauth::AuthMiddleWare;
use hyper::StatusCode;

fn auth_hello() -> Router {
    let (chain, pipeline) = single_pipeline(new_pipeline().add(AuthMiddleWare::default()).build());
    build_router(chain, pipeline, |route| {
        route
            .get("/")
            .to(|state| (state, "Hello Auththorized User!"));
    })
}

#[test]
fn get_401_without_login() {
    let test_server = TestServer::new(auth_hello()).unwrap();
    let response = test_server
        .client()
        .get("http://localhost")
        .perform()
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let headers = response.headers();
    assert_eq!(
        headers.get("WWW-Authenticate").unwrap().to_str().unwrap(),
        "Basic realm=/"
    );
}

#[test]
fn fail_with_invalid_username_password() {
    let test_server = TestServer::new(auth_hello()).unwrap();
    let response = test_server
        .client()
        .get("http://localhost")
        .with_header("Authorization", "Basic balbalbalba".parse().unwrap())
        .perform()
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.read_body().unwrap();
    assert_eq!(&body[..], b"Auth failed ");
}

#[test]
fn success_with_correct_username_password() {
    let test_server = TestServer::new(auth_hello()).unwrap();
    let response = test_server
        .client()
        .get("http://localhost")
        .with_header("Authorization", "Basic YWRtaW46YWRtaW4=".parse().unwrap())
        .perform()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.read_body().unwrap();
    assert_eq!(&body[..], b"Hello Auththorized User!");
}
