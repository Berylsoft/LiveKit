use serde::{Serialize, Deserialize, de::DeserializeOwned};
use hyper::{
    Client, client::connect::HttpConnector,
    header::{self, HeaderValue, HeaderMap},
    Request, Response, Body,
};
pub use hyper::{Error as HttpError, Result as HttpResult};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};

pub const REFERER: &str = "https://live.bilibili.com/";
pub const API_HOST: &str = "https://api.live.bilibili.com";
pub const WEB_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36";

macro_rules! error_conv_impl {
    ($name:ident, $($variant:ident => $error:ty),*, $(,)?) => {
        #[derive(Debug)]
        pub enum $name {
            $(
                $variant($error),
            )*
            HttpFailure(u16, String),
            RateLimited(String),
            Failure(i32, String),
            PostWithoutAccess,
        }

        $(
            impl From<$error> for $name {
                fn from(err: $error) -> $name {
                    <$name>::$variant(err)
                }
            }
        )*
    };
}

error_conv_impl!(
    RestApiError,
    Network        => HttpError,
    ParseString    => std::string::FromUtf8Error,
    Parse          => serde_json::Error,
    EncodePostBody => serde_urlencoded::ser::Error,
);

impl std::fmt::Display for RestApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RestApiError {}

pub type RestApiResult<Data> = Result<Data, RestApiError>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Access {
    pub uid: u32,
    pub key: String,
    pub csrf: String,
}

fn split_into_kv(pair: &str, pat: char) -> Option<(&str, &str)> {
    // ref: https://doc.servo.org/src/cookie/parse.rs.html#108-111
    match pair.find(pat) {
        Some(i) => Some((&pair[..i], &pair[(i + 1)..])),
        None => None,
    }
}

const K_UID: &str = "DedeUserID";
const K_KEY: &str = "SESSDATA";
const K_CSRF: &str = "bili_jct";

impl Access {
    pub fn from_cookie<Str: AsRef<str>>(cookie: Str) -> Option<Access> {
        macro_rules! seat {
            ($name:tt, $ty:ty) => {
                let mut $name: Option<$ty> = None;
            };
        }

        seat!(uid, u32);
        seat!(key, String);
        seat!(csrf, String);

        for pair in cookie.as_ref().split(';') {
            let (k, v) = split_into_kv(pair.trim(), '=')?;
            let (k, v) = (k.trim(), v.trim());

            macro_rules! occupy {
                ($name:ident) => {{
                    if let Some(_) = &$name { return None };
                    $name = Some(v.parse().ok()?);
                }};
            }

            match k {
                K_UID => occupy!(uid),
                K_KEY => occupy!(key),
                K_CSRF => occupy!(csrf),
                _ => { },
            }
        }

        Some(Access {
            uid: uid?,
            key: key?,
            csrf: csrf?,
        })
    }

    pub fn as_cookie(&self) -> String {
        format!("{}={}; {}={}; {}={}", K_UID, self.uid, K_KEY, self.key, K_CSRF, self.csrf)
    }
}

pub enum RestApiRequestKind {
    BareGet,
    Get,
    Post { form: bool },
}

#[derive(Deserialize)]
pub struct RestApiResponse<Data> { // Data: DeserializeOwned
    pub code: i32,
    pub data: Data,
    pub message: String,
}

pub trait RestApi: Serialize {
    type Response: DeserializeOwned;

    fn kind(&self) -> RestApiRequestKind;
    fn path(&self) -> String;
}

pub type Connector = HttpsConnector<HttpConnector>;

pub type InnerClient = Client<Connector>;

#[derive(Clone)]
pub struct HttpClient {
    client: InnerClient,
    access: Option<Access>,
    proxy: Option<String>,
}

impl HttpClient {
    pub fn build_connector() -> Connector {
        HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_only()
            .enable_http1()
            .build()
    }

    pub fn set_headers(&self, headers: &mut HeaderMap) {
        headers.insert(header::REFERER, HeaderValue::from_static(REFERER));
        headers.insert(header::ORIGIN, HeaderValue::from_static(REFERER));
        headers.insert(header::USER_AGENT, HeaderValue::from_static(WEB_USER_AGENT));
        if let Some(_access) = &self.access {
            headers.insert(header::COOKIE, {
                let mut cookie = HeaderValue::from_str(_access.as_cookie().as_str()).unwrap();
                cookie.set_sensitive(true);
                cookie
            });
        }
    }

    pub fn new(access: Option<Access>, proxy: Option<String>) -> Self {
        Self {
            client: Client::builder().build(HttpClient::build_connector()),
            access,
            proxy,
        }
    }

    pub fn new_bare() -> Self {
        HttpClient::new(None, None)
    }

    #[inline]
    pub async fn get(&self, url: String) -> HttpResult<Response<Body>> {
        self.client.get(url.parse().unwrap()).await
    }

    #[inline]
    pub fn url<Str: AsRef<str>>(&self, path: Str) -> String {
        format!("{}{}", match &self.proxy {
            None => API_HOST,
            Some(proxy) => proxy.as_str(),
        }, path.as_ref())
    }

    pub fn csrf(&self) -> RestApiResult<&str> {
        match &self.access {
            Some(_access) => Ok(_access.csrf.as_str()),
            None => Err(RestApiError::PostWithoutAccess),
        }
    }

    pub async fn call<Req: RestApi>(&self, req: &Req) -> RestApiResult<Req::Response> {
        let req = match req.kind() {
            RestApiRequestKind::BareGet => {
                Request::get(self.url(req.path())).body(Body::empty())
            },
            RestApiRequestKind::Get => {
                let mut _req = Request::get(self.url(req.path()));
    
                let headers = _req.headers_mut().unwrap();
                self.set_headers(headers);

                _req.body(Body::empty())
            },
            RestApiRequestKind::Post { form } => {
                let csrf = self.csrf()?;

                let body = if form {
                    format!(
                        "{}&csrf={}&csrf_token={}",
                        serde_urlencoded::to_string(req)?,
                        csrf,
                        csrf
                    )
                } else {
                    format!(
                        "csrf={}&csrf_token={}",
                        csrf,
                        csrf
                    )
                };

                let mut _req = Request::post(self.url(req.path()));

                let headers = _req.headers_mut().unwrap();
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/x-www-form-urlencoded"),
                );
                self.set_headers(headers);

                _req.body(Body::from(body))
            },
        }.unwrap( );
        let resp = self.client.request(req).await?;
        let status = resp.status().as_u16();
        let bytes = hyper::body::to_bytes(resp.into_body()).await?;
        if status != 200 { return Err(RestApiError::HttpFailure(status, hex::encode(bytes))) };
        let text = std::str::from_utf8(bytes.as_ref()).unwrap( );
        let parsed: RestApiResponse<Req::Response> = serde_json::from_str(text)?;
        match parsed.code {
            0 => Ok(parsed.data),
            412 => Err(RestApiError::RateLimited(text.to_owned())),
            code => Err(RestApiError::Failure(code, text.to_owned())),
        }
    }

    pub fn clone_raw(&self) -> InnerClient {
        self.client.clone()
    }
}
