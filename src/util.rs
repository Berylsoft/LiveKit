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

pub mod rest {
    use serde::{Deserialize, de::DeserializeOwned};
    use reqwest::{Client, StatusCode};
    use crate::config::VERSION;

    pub struct RestConfig {
        pub host: Option<String>,
        pub access_token: Option<String>,
        pub emulate_browser: bool,
    }

    #[derive(Deserialize)]
    pub struct RestApiResponse<Data> {
        pub code: i32,
        pub data: Data,
        pub message: String,
    }

    pub async fn call<Data>(url: String) -> Result<Data, String>
    where
        Data: DeserializeOwned,
    {
        let client = Client::builder().user_agent(VERSION).build().unwrap();
        let resp = client.get(format!("https://api.live.bilibili.com{}", url)).header("Cookie", "new_value").send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp = resp.text().await.unwrap();
        let resp: RestApiResponse<Data> = serde_json::from_str(resp.as_str()).unwrap();
        match resp.code {
            0 => Ok(resp.data),
            _ => Err(resp.message),
        }
    }
}
