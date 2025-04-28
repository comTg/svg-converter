use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};

/// 从Base64编码字符串中提取实际数据部分
pub fn extract_base64_data(data_url: &str) -> Result<Vec<u8>> {
    if let Some(pos) = data_url.find(";base64,") {
        let base64_data = &data_url[pos + 8..];
        Ok(general_purpose::STANDARD.decode(base64_data)?)
    } else {
        Err(anyhow!("无法找到base64数据部分"))
    }
}

/// 将二进制数据编码为Base64的Data URL
pub fn encode_to_base64_data_url(data: &[u8], mime_type: &str) -> String {
    let base64 = general_purpose::STANDARD.encode(data);
    format!("data:{};base64,{}", mime_type, base64)
} 