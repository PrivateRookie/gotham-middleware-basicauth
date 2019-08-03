use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_response;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::{FromState, State};
use hyper::header::HeaderMap;
use hyper::{StatusCode, Uri};
use std::io;

#[derive(Clone)]
pub struct AuthMiddleWare {
    userlist: Vec<String>,
    scopes: Vec<String>,
}

impl AuthMiddleWare {
    pub fn new(userlist: Vec<String>, scopes: Vec<String>) -> AuthMiddleWare {
        AuthMiddleWare { userlist, scopes }
    }

    pub fn default() -> AuthMiddleWare {
        AuthMiddleWare {
            userlist: vec!["admin:admin".to_owned()],
            scopes: vec!["/".to_owned()],
        }
    }

    fn auth(&self, code: &str) -> bool {
        match base64::decode(code) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(user_pass) => self.userlist.contains(&user_pass),
                Err(_) => false,
            },
            Err(_) => false,
        }
    }

    fn inside_scopes(&self, uri: &Uri) -> bool {
        let path = uri.path();
        self.scopes
            .clone()
            .into_iter()
            .any(|scope| path.starts_with(&scope[..]))
    }
}

impl NewMiddleware for AuthMiddleWare {
    type Instance = AuthMiddleWare;

    fn new_middleware(&self) -> io::Result<Self::Instance> {
        Ok(self.clone())
    }
}

impl Middleware for AuthMiddleWare {
    fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture>,
    {
        let uri = Uri::borrow_from(&state);
        match self.inside_scopes(uri) {
            true => {
                let header = HeaderMap::borrow_from(&state);
                match header.get("Authorization") {
                    Some(auth) => match auth.to_str() {
                        Ok(auth_info) => {
                            let codes = auth_info.split(" ").collect::<Vec<&str>>();
                            match self.auth(codes[1]) {
                                true => chain(state),
                                false => {
                                    let body = format!("Auth failed ");
                                    auth_error(state, body)
                                }
                            }
                        }
                        Err(e) => {
                            let body = format!("Invalid Auth header: {}", e);
                            auth_error(state, body)
                        }
                    },
                    None => {
                        let body = String::from("Unauthorized");
                        auth_error(state, body)
                    }
                }
            }
            false => chain(state),
        }
    }
}

fn auth_error(state: State, body: String) -> Box<HandlerFuture> {
    let mut resp = create_response(&state, StatusCode::UNAUTHORIZED, mime::TEXT_PLAIN, body);
    let headers = resp.headers_mut();
    let path = Uri::borrow_from(&state).path();
    headers.insert(
        "WWW-Authenticate",
        format!("Basic realm={}", path).parse().unwrap(),
    );
    Box::new(futures::future::ok((state, resp)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::pipeline::new_pipeline;
    use gotham::pipeline::single::single_pipeline;
    use gotham::router::builder::*;
    use gotham::router::Router;
    use gotham::test::TestServer;
    use hyper::StatusCode;

    fn auth_hello() -> Router {
        let (chain, pipeline) =
            single_pipeline(new_pipeline().add(AuthMiddleWare::default()).build());
        build_router(chain, pipeline, |route| {
            route
                .get("/")
                .to(|state| (state, "Hello Auththorized User!"));
        })
    }

    fn scoped_server() -> TestServer {
        let middleware = AuthMiddleWare {
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
    fn receive_401_without_login() {
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
    fn login_failed_invalid_username_password() {
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
    fn login_success() {
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

    #[test]
    fn visit_unscoped_index() {
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
    fn visit_unscoped_subpath() {
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
    fn visit_scoped_page_unauthorize() {
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
    fn visit_scoped_subpath_unauthorize() {
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

        let body = response.read_body().unwrap();
        assert_eq!(&body[..], b"Unauthorized");
    }

    #[test]
    fn visit_scoped_page_authorize() {
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
    fn visit_scoped_subpath_authorize() {
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
}
