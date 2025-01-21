use std::fs::{File};
use std::io::{self, Write, Read};
use base64::{engine::general_purpose::STANDARD};
use base64::Engine;
use openssl::pkey::PKey;
use openssl::ec::{EcKey, EcPoint, EcGroup};
use openssl::nid::Nid;
use chrono::Local;
fn encode_pem(public_key: &[u8]) -> String {
    if public_key.len() != 65 {
        return String::from("Invalid public key length");
    }
    // 构建ASN.1 DER编码的公钥
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let mut ctx = openssl::bn::BigNumContext::new().unwrap();
    // 解析公钥
    let point = EcPoint::from_bytes(&group, &public_key, &mut ctx).unwrap();
    let ec_key = EcKey::from_public_key(&group, &point).unwrap();
    let pkey = PKey::from_ec_key(ec_key).unwrap();
    let der = pkey.public_key_to_der().unwrap();

    // 将DER编码的公钥转换为PEM格式
    let pem = STANDARD.encode(&der);
    format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        pem
    )
}

/// 保存PEM格式的公钥到文件
pub fn save_pem_to_file(public_key: &[u8], file_path: &str) -> io::Result<()> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/public_key_{}.pem", file_path, timestamp);
    let pem = encode_pem(public_key);
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
    let pem = pem.replace("-----BEGIN PUBLIC KEY-----", "")
                .replace("-----END PUBLIC KEY-----", "")
                .replace("\n", "")
                .replace("\r", "");

    // 解码Base64
    let der= STANDARD.decode(pem).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // 将DER解码为原始公钥
    let pkey = PKey::public_key_from_der(&der).unwrap();
    let ec_key = pkey.ec_key().unwrap();
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let point = ec_key.public_key();
    let mut ctx = openssl::bn::BigNumContext::new().unwrap();
    let public_key = point.to_bytes(&group, openssl::ec::PointConversionForm::UNCOMPRESSED, &mut ctx)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(public_key)
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