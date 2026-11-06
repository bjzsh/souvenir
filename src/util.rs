/// `?` for `const fn`: return the error, or evaluate to the value.
macro_rules! tri {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => return Err(e),
        }
    };
}

/// `bytes` as a `&str`, for bytes the caller has built from ASCII only
/// (prefix letters, the separator and the Base32 alphabet).
#[inline]
pub(crate) const fn ascii(bytes: &[u8]) -> &str {
    match core::str::from_utf8(bytes) {
        Ok(s) => s,
        // Unreachable: ASCII is always valid UTF-8.
        Err(_) => "",
    }
}
