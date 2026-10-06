//! Compile-time string obfuscation using XOR.
//! Usage: let secret = obf!("http://evil.com");
//! println!("{}", secret); // Decrypts at runtime

macro_rules! obf {
    ($str:expr) => {{
        let s: &str = $str;
        let key: u8 = 0x42; // Simple XOR key
        let mut buf: Vec<u8> = s.bytes().map(|b| b ^ key).collect();
        String::from_utf8(buf).unwrap()
    }};
}

// Use pub(crate) to make it available to all modules in the crate
pub(crate) use obf;