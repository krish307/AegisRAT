macro_rules! obf {
    ($str:expr) => {{
        let s: &str = $str;
        let key: u8 = 0x42;
        let mut buf: Vec<u8> = s.bytes().map(|b| b ^ key).collect();
        String::from_utf8(buf).unwrap()
    }};
}

pub(crate) use obf;
