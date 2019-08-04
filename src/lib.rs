use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_response;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::{FromState, State};
use hyper::header::HeaderMap;
use hyper::{StatusCode, Uri};
use std::io;

#[derive(Clone)]
pub struct AuthMiddleWare {
    pub userlist: Vec<String>,
    pub scopes: Vec<String>,
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
