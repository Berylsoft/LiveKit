use serde::{Serialize, Deserialize, de::DeserializeOwned};
use reqwest::{Client, header, Response, IntoUrl};

pub const REFERER: &str = "https://live.bilibili.com/";
pub const API_HOST: &str = "https://api.live.bilibili.com";
pub const WEB_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36";

#[derive(Debug)]
pub enum RestApiError {
    Network(reqwest::Error),
    HttpFailure(u16, String),
    Parse(serde_json::Error),
    RateLimited(String),
    Failure(i32, String),
}

impl From<reqwest::Error> for RestApiError {
    fn from(err: reqwest::Error) -> RestApiError {
        RestApiError::Network(err)
    }
}

impl From<serde_json::Error> for RestApiError {
    fn from(err: serde_json::Error) -> RestApiError {
        RestApiError::Parse(err)
    }
}

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

impl Access {
    pub fn from_cookie<T: AsRef<str>>(cookie: T) -> Option<Access> {
        macro_rules! seat {
            ($name:tt, $ty:ty) => {
                let mut $name: Option<$ty> = None;
            };
        }

        seat!(uid, u32);
        seat!(key, String);
        seat!(csrf, String);

        for pair in cookie.as_ref().split(";") {
            let (k, v) = split_into_kv(pair.trim(), '=')?;
            let (k, v) = (k.trim(), v.trim());

            macro_rules! occupy {
                ($name:ident) => {{
                    if let Some(_) = &$name { return None };
                    $name = Some(v.parse().ok()?);
                }};
            }

            match k {
                "DedeUserID" => occupy!(uid),
                "SESSDATA" => occupy!(key),
                "bili_jct" => occupy!(csrf),
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
        format!("DedeUserID={}; SESSDATA={}; bili_jct={}", self.uid, self.key, self.csrf)
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
    host: String,
    client: Client,
    access: Option<Access>,
}

impl HttpClient {
    pub async fn new(access: Option<Access>, api_proxy: Option<String>) -> Self {
        let mut headers = header::HeaderMap::new();
        let referer = header::HeaderValue::from_str(REFERER).unwrap();
        headers.insert(header::REFERER, referer);
        if let Some(_access) = &access {
            let mut cookie = header::HeaderValue::from_str(_access.as_cookie().as_str()).unwrap();
            cookie.set_sensitive(true);
            headers.insert(header::COOKIE, cookie);
        }
        let host = match api_proxy {
            None => API_HOST.to_owned(),
            Some(host) => host.clone(),
        };
        Self {
            host,
            client: Client::builder().user_agent(WEB_USER_AGENT).default_headers(headers).build().unwrap(),
            access,
        }
    }

    pub async fn new_bare() -> Self {
        Self {
            host: API_HOST.to_owned(),
            client: Client::new(),
            access: None,
        }
    }

    #[inline]
    pub async fn get<T: IntoUrl>(&self, url: T) -> reqwest::Result<Response> {
        self.client.get(url).send().await
    }

    #[inline]
    pub fn url<T: AsRef<str>>(&self, path: T) -> String {
        format!("{}{}", self.host, path.as_ref())
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

    pub async fn call<Data: DeserializeOwned, T: AsRef<str>>(&self, path: T) -> RestApiResult<Data>
    {
        self.proc_call(self.client.get(self.url(path)).send().await?).await
    }

    pub fn clone_raw(&self) -> Client {
        self.client.clone()
    }
}
