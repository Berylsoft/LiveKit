pub struct Timestamp(i64); // u64?

#[cfg(feature = "package")]
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

    pub fn from_bytes(raw: [u8; 8]) -> Timestamp {
        Timestamp(i64::from_be_bytes(raw))
    }
}

#[cfg(feature = "schema")]
pub mod json {
    use serde::de::DeserializeOwned;
    use serde_json::{Value, Result};

    // same as `serde_json::from_value`, but takes reference
    pub fn to<T: DeserializeOwned>(value: &Value) -> Result<T>
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

    /*

    pub fn inline_json<T: DeserializeOwned>(value: &Value) -> Result<T>
    {
        let json: String = to(value)?;
        Ok(serde_json::from_str(json.as_str())?)
    }

    pub fn inline_json_opt<T: DeserializeOwned>(value: &Value) -> Result<Option<T>>
    {
        let json: String = to(value)?;
        if json == "{}" {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_str(json.as_str())?))
        }
    }

    */

    pub fn may_inline_json_opt<T: DeserializeOwned>(value: &Value) -> Result<Option<T>>
    {
        match value.as_str() {
            None => Ok(Some(to(value)?)),
            Some("{}") => Ok(None),
            Some(json) => Ok(Some(serde_json::from_str(json)?))
        }
    }

    pub fn string_opt(value: &Value) -> Result<Option<String>> {
        let string: String = to(value)?;
        if string.is_empty() {
            Ok(None)
        } else {
            Ok(Some(string))
        }
    }

    // todo num_opt
    pub fn u32_opt(value: &Value) -> Result<Option<u32>> {
        let num: u32 = to(value)?;
        if num == 0 {
            Ok(None)
        } else {
            Ok(Some(num))
        }
    }

    pub fn string_u32(value: &Value) -> Result<u32> {
        let string: String = to(value)?;
        Ok(string.parse::<u32>().unwrap())
    }

    pub fn string_color_to_u32(value: &Value) -> Result<u32> {
        if value.is_string() {
            let string: String = to(value)?;
            let string = {
                assert_eq!(string.len(), 7);
                let mut c = string.chars();
                assert_eq!(c.next(), Some('/'));
                format!("00{}", c.as_str())
            };
            let mut buf = [0u8; 4];
            hex::decode_to_slice(string, &mut buf).unwrap();
            Ok(u32::from_be_bytes(buf))
        } else {
            Ok(to(value)?)
        }
    }
}
