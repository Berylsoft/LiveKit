pub async fn call_rest_api<Data>(url: String) -> Option<Data>
where
    Data: serde::de::DeserializeOwned,
{
    use serde_json::from_str as parse_json;
    use reqwest::{get as http_get, StatusCode};
    use crate::api_schema::rest::RestApiResponse;

    let resp = http_get(url.as_str()).await.unwrap();
    match resp.status() {
        StatusCode::OK => (),
        _ => return None,
    }
    let resp = resp.text().await.unwrap();
    let resp: RestApiResponse<Data> = parse_json(resp.as_str()).unwrap();
    match resp.code {
        0 => Some(resp.data),
        _ => None, // Err(resp.message)
    }
}
