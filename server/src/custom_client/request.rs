use std::fs::{self, OpenOptions};
use std::io::BufReader;
use std::sync::{Arc, LazyLock};

use bytes::Bytes;
use cookie_store::CookieStore;
use futures::{Stream, StreamExt};
use kreqo_core::cookies_path;
use reqwest::Body;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::multipart::Form;
pub use reqwest::{Client, Method, Request, Url};
use reqwest_cookie_store::CookieStoreMutex;
use server_fn::client::get_server_url;
use server_fn::error::{FromServerFnError, IntoAppError, ServerFnErrorErr};
use server_fn::request::ClientReq;

pub(crate) static COOKIE_STORE: LazyLock<Arc<CookieStoreMutex>> = LazyLock::new(|| {
    let cookie_store = {
        let path = cookies_path();
        if let Ok(file) = fs::File::open(path).map(BufReader::new) {
            cookie_store::serde::json::load(file).unwrap_or_default()
        } else {
            CookieStore::new()
        }
    };
    Arc::new(CookieStoreMutex::new(cookie_store))
});

pub(crate) static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .cookie_provider(Arc::clone(&COOKIE_STORE))
        .build()
        .unwrap()
});

pub fn save_cookies() -> anyhow::Result<()> {
    if let Ok(cookie_store) = COOKIE_STORE.lock() {
        let path = cookies_path();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        cookie_store::serde::json::save(&cookie_store, &mut file).unwrap();
    }
    Ok(())
}

pub struct CustomRequest(pub Request);

impl From<Request> for CustomRequest {
    fn from(value: Request) -> Self {
        Self(value)
    }
}

impl<E> ClientReq<E> for CustomRequest
where
    E: FromServerFnError,
{
    type FormData = Form;

    fn try_new_req_query(
        path: &str,
        content_type: &str,
        accepts: &str,
        query: &str,
        method: Method,
    ) -> Result<Self, E> {
        let url = format!("{}{}", get_server_url(), path);
        let mut url = Url::try_from(url.as_str())
            .map_err(|e| E::from_server_fn_error(ServerFnErrorErr::Request(e.to_string())))?;
        url.set_query(Some(query));
        let req = match method {
            Method::GET => CLIENT.get(url),
            Method::DELETE => CLIENT.delete(url),
            Method::HEAD => CLIENT.head(url),
            Method::POST => CLIENT.post(url),
            Method::PATCH => CLIENT.patch(url),
            Method::PUT => CLIENT.put(url),
            m => {
                return Err(E::from_server_fn_error(
                    ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                ));
            }
        }
        .header(CONTENT_TYPE, content_type)
        .header(ACCEPT, accepts)
        .build()
        .map_err(|e| E::from_server_fn_error(ServerFnErrorErr::Request(e.to_string())))?;
        Ok(req.into())
    }

    fn try_new_req_text(
        path: &str,
        content_type: &str,
        accepts: &str,
        body: String,
        method: Method,
    ) -> Result<Self, E> {
        let url = format!("{}{}", get_server_url(), path);
        let req = match method {
            Method::POST => CLIENT.post(url),
            Method::PUT => CLIENT.put(url),
            Method::PATCH => CLIENT.patch(url),
            m => {
                return Err(E::from_server_fn_error(
                    ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                ));
            }
        }
        .header(CONTENT_TYPE, content_type)
        .header(ACCEPT, accepts)
        .body(body)
        .build()
        .map_err(|e| ServerFnErrorErr::Request(e.to_string()).into_app_error())?;
        Ok(req.into())
    }

    fn try_new_req_bytes(
        path: &str,
        content_type: &str,
        accepts: &str,
        body: Bytes,
        method: Method,
    ) -> Result<Self, E> {
        let url = format!("{}{}", get_server_url(), path);
        let req = match method {
            Method::POST => CLIENT.post(url),
            Method::PATCH => CLIENT.patch(url),
            Method::PUT => CLIENT.put(url),
            m => {
                return Err(E::from_server_fn_error(
                    ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                ));
            }
        }
        .header(CONTENT_TYPE, content_type)
        .header(ACCEPT, accepts)
        .body(body)
        .build()
        .map_err(|e| ServerFnErrorErr::Request(e.to_string()).into_app_error())?;
        Ok(req.into())
    }

    fn try_new_req_multipart(
        path: &str,
        accepts: &str,
        body: Self::FormData,
        method: Method,
    ) -> Result<Self, E> {
        let req = match method {
            Method::POST => CLIENT.post(path),
            Method::PUT => CLIENT.put(path),
            Method::PATCH => CLIENT.patch(path),
            m => {
                return Err(E::from_server_fn_error(
                    ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                ));
            }
        }
        .header(ACCEPT, accepts)
        .multipart(body)
        .build()
        .map_err(|e| ServerFnErrorErr::Request(e.to_string()).into_app_error())?;
        Ok(req.into())
    }

    fn try_new_req_form_data(
        path: &str,
        accepts: &str,
        content_type: &str,
        body: Self::FormData,
        method: Method,
    ) -> Result<Self, E> {
        let req = match method {
            Method::POST => CLIENT.post(path),
            Method::PATCH => CLIENT.patch(path),
            Method::PUT => CLIENT.put(path),
            m => {
                return Err(E::from_server_fn_error(
                    ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                ));
            }
        }
        .header(CONTENT_TYPE, content_type)
        .header(ACCEPT, accepts)
        .multipart(body)
        .build()
        .map_err(|e| ServerFnErrorErr::Request(e.to_string()).into_app_error())?;
        Ok(req.into())
    }

    fn try_new_req_streaming(
        path: &str,
        accepts: &str,
        content_type: &str,
        body: impl Stream<Item = Bytes> + Send + 'static,
        method: Method,
    ) -> Result<Self, E> {
        let url = format!("{}{}", get_server_url(), path);
        let body =
            Body::wrap_stream(body.map(|chunk| Ok(chunk) as Result<Bytes, ServerFnErrorErr>));
        let req = match method {
            Method::POST => CLIENT.post(url),
            Method::PUT => CLIENT.put(url),
            Method::PATCH => CLIENT.patch(url),
            m => {
                return Err(E::from_server_fn_error(
                    ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                ));
            }
        }
        .header(CONTENT_TYPE, content_type)
        .header(ACCEPT, accepts)
        .body(body)
        .build()
        .map_err(|e| ServerFnErrorErr::Request(e.to_string()).into_app_error())?;
        Ok(req.into())
    }
}
