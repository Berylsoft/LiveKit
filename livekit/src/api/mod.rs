use serde::{Deserialize, de::DeserializeOwned};
use reqwest::{Client, StatusCode};
// use crate::config::VERSION;

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
    // .user_agent(VERSION)
    let client = Client::builder().build().unwrap();
    // .header("Cookie", "new_value")
    let resp = client.get(format!("https://api.live.bilibili.com{}", url)).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = resp.text().await.unwrap();
    let resp: RestApiResponse<Data> = serde_json::from_str(resp.as_str()).unwrap();
    match resp.code {
        0 => Ok(resp.data),
        _ => Err(resp.message),
    }
}

pub mod room;
