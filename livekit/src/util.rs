pub mod compress {
    pub use inflate::inflate_bytes as inflate;

    use std::io::Cursor;
    use brotli_decompressor::BrotliDecompress;

    pub fn de_brotli(raw: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut decoded = Vec::new();
        BrotliDecompress(&mut Cursor::new(raw), &mut Cursor::new(&mut decoded))?;
        Ok(decoded)
    }
}

pub struct Timestamp(i64); // u64?

impl Timestamp {
    pub fn now() -> Self {
        Timestamp(chrono::Utc::now().timestamp_millis())
    }

    #[inline]
    pub fn digits(&self) -> i64 {
        self.0
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.digits().to_be_bytes()
    }
}

pub mod vec {
    pub fn concat<T>(mut a: Vec<T>, mut b: Vec<T>) -> Vec<T> {
        a.append(&mut b);
        a
    }
}

pub mod http {
    use serde::{Deserialize, de::DeserializeOwned};
    use reqwest::{Client, StatusCode, header, Response};
    use crate::config::{WEB_USER_AGENT, API_HOST, REFERER, CommonConfig};

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
        pub async fn new(common_config: &CommonConfig) -> Self {
            let mut headers = header::HeaderMap::new();
            let referer = header::HeaderValue::from_str(REFERER).unwrap();
            headers.insert(header::REFERER, referer);
            if let Some(token) = &common_config.access_token {
                let mut cookie = header::HeaderValue::from_str(token).unwrap();
                cookie.set_sensitive(true);
                headers.insert(header::COOKIE, cookie);
            }
            let host = match &common_config.api_proxy {
                None => API_HOST.to_string(),
                Some(host) => host.clone(),
            };
            Self {
                host,
                client: Client::builder().user_agent(WEB_USER_AGENT).default_headers(headers).build().unwrap(),
            }
        }

        pub async fn new_bare() -> Self {
            Self {
                host: API_HOST.to_string(),
                client: Client::builder().build().unwrap(),
            }
        }

        pub async fn get(&self, url: String) -> Response {
            self.client.get(url).send().await.unwrap()
        }

        pub async fn call<Data>(&self, url: String) -> Result<Data, String>
        where
            Data: DeserializeOwned,
        {
            let resp = self.get(format!("{}{}", self.host, url)).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let resp = resp.text().await.unwrap();
            let resp: RestApiResponse<Data> = serde_json::from_str(resp.as_str()).unwrap();
            match resp.code {
                0 => Ok(resp.data),
                _ => Err(resp.message),
            }
        }
    }
}
