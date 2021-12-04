use serde::{Deserialize, de::DeserializeOwned};
use reqwest::{Client, StatusCode, header, Response};

pub const REFERER: &str = "https://live.bilibili.com/";
pub const API_HOST: &str = "https://api.live.bilibili.com";
pub const WEB_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36";

#[derive(Debug)]
pub enum RestApiError {
    Network(reqwest::Error),
    HttpFailure(StatusCode),
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
}

impl HttpClient {
    pub async fn new(access_token: Option<String>, api_proxy: Option<String>) -> Self {
        let mut headers = header::HeaderMap::new();
        let referer = header::HeaderValue::from_str(REFERER).unwrap();
        headers.insert(header::REFERER, referer);
        if let Some(token) = access_token {
            let mut cookie = header::HeaderValue::from_str(token.as_str()).unwrap();
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
        }
    }

    pub async fn new_bare() -> Self {
        Self {
            host: API_HOST.to_owned(),
            client: Client::new(),
        }
    }

    pub async fn get(&self, url: String) -> Result<Response, reqwest::Error> {
        self.client.get(url).send().await
    }

    pub async fn call<Data>(&self, url: String) -> RestApiResult<Data>
    where
        Data: DeserializeOwned,
    {
        let resp = self.get(format!("{}{}", self.host, url)).await?;
        match resp.status() {
            StatusCode::OK => { },
            status => return Err(RestApiError::HttpFailure(status)),
        }
        let text = resp.text().await?;
        let parsed: RestApiResponse<Data> = serde_json::from_str(text.as_str())?;
        match parsed.code {
            0 => Ok(parsed.data),
            412 => Err(RestApiError::RateLimited(text)),
            code => Err(RestApiError::Failure(code, text)),
        }
    }

    pub fn clone_raw(&self) -> Client {
        self.client.clone()
    }
}
