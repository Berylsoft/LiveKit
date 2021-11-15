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

        pub fn clone_raw(&self) -> Client {
            self.client.clone()
        }
    }
}

pub mod json {
    use serde::de::DeserializeOwned;
    use serde_json::{Value, Result};

    // same as `serde_json::from_value`, but takes reference
    pub fn to<T>(value: &Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        T::deserialize(value)
    }

    pub fn numbool(value: &Value) -> Result<bool> {
        let num: u8 = to(value)?;
        if num == 0 {
            Ok(false)
        } else if num == 1 {
            Ok(true)
        } else {
            panic!()
        }
    }

    pub fn inline_json<T>(value: &Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let json: String = to(value)?;
        Ok(serde_json::from_str(json.as_str())?)
    }

    pub fn inline_json_opt<T>(value: &Value) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let json: String = to(value)?;
        if json == "{}" {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_str(json.as_str())?))
        }
    }

    pub fn string_opt(value: &Value) -> Result<Option<String>> {
        let string: String = to(value)?;
        if string.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(string))
        }
    }

    // todo num_opt
    pub fn u32_opt(value: &Value) -> Result<Option<u32>>
    {
        let num: u32 = to(value)?;
        if num == 0 {
            Ok(None)
        } else {
            Ok(Some(num))
        }
    }
}
