struct TypeEncodingError(String);
impl<T: std::fmt::Display> From<T> for TypeEncodingError {
    fn from(value: T) -> Self {
        Self(format!("type_encoder: {}", value))
    }
}




struct StringNulEncodingError {}
impl std::fmt::Display for StringNulEncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "string value to nul-encode already contains a null character")
    }
}




struct StringLenencEncodingError {}
impl std::fmt::Display for StringLenencEncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "string value is larger than lenenc can handle")
    }
}




/// Is `value` is `None` the integer is interpreted as `NULL`
pub fn encode_integer_lencenc(value: Option<u64>) -> Vec<u8> {
    let mut encoded: (u8, Vec<u8>) = match value {
        Some(value) => match value {
            0..0xfb => { return vec![value as u8]; }
            0xfb..=0xffff => (0xfc, Vec::from((value as u16).to_le_bytes())),
            0x10000..=0xffffff => (0xfd, Vec::from(
                unsafe { let org_slice: [u8; 4] = (value as u32).to_le_bytes(); &*(org_slice.as_ptr() as *const [u8; 3]) }
            )),
            0x1000000..=0xffffffffffffffff => (0xfe, Vec::from(value.to_le_bytes())),
        },
        None => (0xfb, Vec::new())
    };
    encoded.1.insert(0, encoded.0);
    encoded.1
}




pub fn encode_string_nul(value: String) -> Result<Vec<u8>, StringNulEncodingError> {
    for ch in value.chars() {
        if ch == '\0' { return Err(StringNulEncodingError {}); }
    }
    Ok(Vec::from(value.as_bytes()))
}




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_integer_lencenc_type_null() {
        assert_eq!(encode_integer_lencenc(None), vec![0xfb]);
    }

    #[test]
    fn encode_integer_lencenc_type_0() {
        assert_eq!(encode_integer_lencenc(Some(0xfa)), vec![0xfa]);
    }

    #[test]
    fn encode_integer_lencenc_type_1_01() {
        assert_eq!(encode_integer_lencenc(Some(0xfb)), vec![0xfc, 0xfb, 0x0]);
    }

    #[test]
    fn encode_integer_lencenc_type_2_01() {
        assert_eq!(encode_integer_lencenc(Some(0x010000)), vec![0xfd, 0x0, 0x0, 0x1]);
    }

    #[test]
    fn encode_integer_lencenc_type_2_02() {
        assert_eq!(encode_integer_lencenc(Some(0xffffff)), vec![0xfd, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn encode_integer_lencenc_type_3_01() {
        assert_eq!(encode_integer_lencenc(Some(0x0000000001000000)), vec![0xfe, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0]);
    }
}
