// Generate userp at compile time


fn main() {
    let userp = std::env::var("USERP").expect("USERP must be set");
    let key: u8 = 0x42;

    // XOR obfuscate and encode as hex string to avoid readable patterns in binary
    let obf_bytes: Vec<u8> = userp.bytes().map(|b| b ^ key).collect();
    let hex_string: String = obf_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    let code = format!(
        "pub const USERP_KEY: u8 = 0x{:02x};\n\
         pub const USER_P_OBF_HEX: &[u8] = b\"{}\";",
        key, hex_string
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("userp_secret.rs");
    std::fs::write(&dest_path, code).unwrap();
}

