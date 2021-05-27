use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::test::TestServer;
use gotham_middleware_basicauth::AuthMiddleware;
use gotham::hyper::StatusCode;

fn scoped_server() -> TestServer {
    let middleware: AuthMiddleware = AuthMiddleware {
        userlist: vec!["admin:admin".to_owned()],
        scopes: vec!["/scoped".to_owned()],
    };
    let (chain, pipeline) = single_pipeline(new_pipeline().add(middleware).build());
    let router = build_router(chain, pipeline, |route| {
        route.get("/").to(|state| (state, "Public Page"));
        route.get("/page").to(|state| (state, "Public Page"));
        route.get("/scoped").to(|state| (state, "Private Page"));
        route
            .get("/scoped/page")
            .to(|state| (state, "Private Page"));
    });
    TestServer::new(router).unwrap()
}

#[test]
fn unscoped_index() {
    let test_server = scoped_server();

    let response = test_server
        .client()
        .get("http://localhost")
        .perform()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.read_body().unwrap();
    assert_eq!(&body[..], b"Public Page");
}

#[test]
fn unscoped_subpath() {
    let test_server = scoped_server();

    let response = test_server
        .client()
        .get("http://localhost/page")
        .perform()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.read_body().unwrap();
    assert_eq!(&body[..], b"Public Page");
}

#[test]
fn scoped_page_unauthorize() {
    let test_server = scoped_server();
    let response = test_server
        .client()
        .get("http://localhost/scoped")
        .perform()
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let headers = response.headers();
    assert_eq!(
        headers.get("WWW-Authenticate").unwrap().to_str().unwrap(),
        "Basic realm=/scoped"
    );

    let body = response.read_body().unwrap();
    assert_eq!(&body[..], b"Unauthorized");
}

#[test]
fn scoped_subpath_unauthorize() {
    let test_server = scoped_server();
    let response = test_server
        .client()
        .get("http://localhost/scoped/page")
        .perform()
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let headers = response.headers();
    assert_eq!(
        headers.get("WWW-Authenticate").unwrap().to_str().unwrap(),
        "Basic realm=/scoped/page"
    );

    let body: Vec<u8> = response.read_body().unwrap();
    assert_eq!(&body[..], b"Unauthorized");
}

#[test]
fn scoped_page_authorize() {
    let test_server = scoped_server();
    let response = test_server
        .client()
        .get("http://localhost/scoped")
        .with_header("Authorization", "Basic YWRtaW46YWRtaW4=".parse().unwrap())
        .perform()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.read_body().unwrap();
    assert_eq!(&body[..], b"Private Page");
}

#[test]
fn scoped_subpath_authorize() {
    let test_server = scoped_server();
    let response = test_server
        .client()
        .get("http://localhost/scoped/page")
        .with_header("Authorization", "Basic YWRtaW46YWRtaW4=".parse().unwrap())
        .perform()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.read_body().unwrap();
    assert_eq!(&body[..], b"Private Page");
}
