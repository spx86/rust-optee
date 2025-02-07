use std::fs::{File};
use std::io::{self, Write, Read};
use base64::{engine::general_purpose::STANDARD};
use base64::Engine;
use openssl::pkey::PKey;
use openssl::ec::{EcKey, EcPoint, EcGroup};
use openssl::nid::Nid;
use chrono::Local;
fn encode_aes_key(aes_key: &[u8]) -> String {
    // AES 密钥必须是 16 字节（128 bit）、24 字节（192 bit）或 32 字节（256 bit）
    if aes_key.len() != 16 && aes_key.len() != 24 && aes_key.len() != 32 {
        return String::from("Invalid AES key length");
    }

    // 将 AES 密钥编码为 Base64 格式
    let base64_key = STANDARD.encode(aes_key);

    // 格式化为 PEM 格式
    format!(
        "-----BEGIN AES KEY-----\n{}\n-----END AES KEY-----",
        base64_key
    )
}

/// 保存PEM格式的公钥到文件
pub fn save_pem_to_file(public_key: &[u8], file_path: &str) -> io::Result<()> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/public_key_{}.pem", file_path, timestamp);
    let pem = encode_aes_key(public_key);
    let mut file = File::create(file_path)?;
    file.write_all(pem.as_bytes())?;
    Ok(())
}

/// 从文件中读取PEM格式的公钥并解码为原始公钥
pub fn read_pem_from_file(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut pem = String::new();
    file.read_to_string(&mut pem)?;

    // 提取Base64部分
    let pem = pem.replace("-----BEGIN AES KEY-----", "")
                .replace("-----END AES KEY-----", "")
                .replace("\n", "")
                .replace("\r", "");

    // 将解码为原始密钥
    let aes_key =  STANDARD.decode(pem).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(aes_key)
}

pub fn save_sign_to_file(sign: &[u8], file_path: &str) -> io::Result<()> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/signature_{}.dat", file_path, timestamp);
    let mut file = File::create(file_path)?;
    file.write_all(sign)?;
    Ok(())
}

pub fn read_sign_from_file(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut sign = Vec::new();
    file.read_to_end(&mut sign)?;
    Ok(sign)
}