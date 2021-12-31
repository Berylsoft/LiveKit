use serde::{Serialize, Deserialize, de::DeserializeOwned};
use reqwest::{Client, header::{self, HeaderValue, HeaderMap}, Response, IntoUrl};
pub use reqwest::Error as ReqwestError;

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
    Network        => reqwest::Error,
    Parse          => serde_json::Error,
    EncodePostBody => serde_urlencoded::ser::Error,
);

pub type RestApiResult<Data> = Result<Data, RestApiError>;

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Deserialize)]
pub struct RestApiResponse<Data> {
    pub code: i32,
    pub data: Data,
    pub message: String,
}

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    access: Option<Access>,
    proxy: Option<String>,
}

impl HttpClient {
    pub async fn new(access: Option<Access>, proxy: Option<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(header::REFERER, HeaderValue::from_static(REFERER));
        headers.insert(header::ORIGIN, HeaderValue::from_static(REFERER));
        headers.insert(header::USER_AGENT, HeaderValue::from_static(WEB_USER_AGENT));
        if let Some(_access) = &access {
            headers.insert(header::COOKIE, {
                let mut cookie = HeaderValue::from_str(_access.as_cookie().as_str()).unwrap();
                cookie.set_sensitive(true);
                cookie
            });
        }
        Self {
            client: Client::builder().default_headers(headers).build().unwrap(),
            access,
            proxy,
        }
    }

    pub async fn new_bare() -> Self {
        Self {
            client: Client::new(),
            access: None,
            proxy: None,
        }
    }

    #[inline]
    pub async fn get<Url: IntoUrl>(&self, url: Url) -> reqwest::Result<Response> {
        self.client.get(url).send().await
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

    pub async fn proc_call<Data: DeserializeOwned>(&self, resp: Response) -> RestApiResult<Data>
    {
        let status = resp.status().as_u16();
        let text = resp.text().await?;
        if status != 200 { return Err(RestApiError::HttpFailure(status, text)) };
        let parsed: RestApiResponse<Data> = serde_json::from_str(text.as_str())?;
        match parsed.code {
            0 => Ok(parsed.data),
            412 => Err(RestApiError::RateLimited(text)),
            code => Err(RestApiError::Failure(code, text)),
        }
    }

    pub async fn call<Data: DeserializeOwned, Str: AsRef<str>>(&self, path: Str) -> RestApiResult<Data>
    {
        self.proc_call(self.client.get(self.url(path)).send().await?).await
    }

    pub async fn call_post<Data: DeserializeOwned, Form: Serialize, Str: AsRef<str>>(&self, path: Str, form: Option<Form>) -> RestApiResult<Data>
    {
        let csrf = self.csrf()?;

        let body = match form {
            Some(_form) => format!(
                "{}&csrf={}&csrf_token={}",
                serde_urlencoded::to_string(_form)?,
                csrf,
                csrf
            ),
            None => format!(
                "csrf={}&csrf_token={}",
                csrf,
                csrf
            )
        };

        self.proc_call(
            self.client
                .post(self.url(path))
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/x-www-form-urlencoded"),
                )
                .body(body)
                .send().await?
        ).await
    }

    pub fn clone_raw(&self) -> Client {
        self.client.clone()
    }
}
