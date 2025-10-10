use serde::{self, Deserialize, Deserializer, Serializer};
use windows::Win32::Foundation::HWND;

pub fn serialize<S>(hwnd: &Option<HWND>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match hwnd {
        Some(hwnd) => {
            let addr = hwnd.0 as usize;
            let hex = format!("0x{:X}", addr);
            serializer.serialize_str(&hex)
        }
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<HWND>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                let hex_str = trimmed.trim_start_matches("0x");
                usize::from_str_radix(hex_str, 16)
                    .map(|val| {
                        let ptr = val as *mut core::ffi::c_void;
                        Some(HWND(ptr))
                    })
                    .map_err(serde::de::Error::custom)
            }
        }
        None => Ok(None),
    }
}
