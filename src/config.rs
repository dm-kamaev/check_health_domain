
use crate::{obf_str, private_data::{SecretBuf}};


const EMAIL: [u8; 23] = obf_str!("qwertyzxcv526@gmail.com", USERP_KEY);

include!(concat!(env!("OUT_DIR"), "/userp_secret.rs"));

const EMAIL2: [u8; 23] = obf_str!("bukhalkina_anna@mail.ru", USERP_KEY);


pub fn get_first_user() -> SecretBuf<23> {
  SecretBuf::<23>::deobf(&EMAIL, USERP_KEY)
}

pub fn get_second_user() -> SecretBuf<23> {
  SecretBuf::<23>::deobf(&EMAIL2, USERP_KEY)
}

pub fn get_first_userp() -> SecretBuf<16> {
  SecretBuf::<16>::deobf_from_hex(&USER_P_OBF_HEX, USERP_KEY)
}

pub fn get_recipients() -> Vec<String> {
  vec![get_first_user().as_str().to_string(), get_second_user().as_str().to_string()]
}

pub const URL: &str = "https://risk-monitoring.ru";
pub const FALLBACK_URL: &str = "https://yandex.com";

