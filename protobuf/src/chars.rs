#![cfg(feature = "bytes")]

use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;
use std::str;

use bytes::Bytes;

/// Thin wrapper around `Bytes` which guarantees that bytes are valid UTF-8 string.
/// Should be API-compatible to `String`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chars(Bytes);

impl Chars {
    /// New empty object.
    pub const fn new() -> Chars {
        Chars(Bytes::new())
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Try convert from `Bytes`
    pub fn from_bytes(bytes: Bytes) -> Result<Chars, str::Utf8Error> {
        str::from_utf8(&bytes)?;

        Ok(Chars(bytes))
    }

    /// Len in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Self-explanatory
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consume self and return the underlying bytes.
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Self-explanatory
    pub fn as_str(&self) -> &str {
        self
    }
}

// Chars can be used as the key in a HashMap in a proto map. Because Chars is a wrapper around
// Bytes, we cannot derive Hash on bytes, because hashing Bytes/u8 is not the same as hashing str.
// So here we make sure when we hash we are hashing the str otherwise we get very confusing
// results.
impl Hash for Chars {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let str_self: &str = self;
        str_self.hash(state);
    }
}

impl From<&'static str> for Chars {
    fn from(value: &'static str) -> Self {
        Chars(Bytes::from_static(value.as_bytes()))
    }
}

impl From<String> for Chars {
    fn from(src: String) -> Chars {
        Chars(Bytes::from(src))
    }
}

impl Into<String> for Chars {
    fn into(self) -> String {
        // This is safe because `Chars` is guaranteed to store a valid UTF-8 string
        unsafe { String::from_utf8_unchecked(self.0.as_ref().to_owned()) }
    }
}

impl Default for Chars {
    fn default() -> Self {
        Chars::new()
    }
}

impl Deref for Chars {
    type Target = str;

    fn deref(&self) -> &str {
        // This is safe because `Chars` is guaranteed to store a valid UTF-8 string
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<T> AsRef<T> for Chars
where
    T: ?Sized,
    <Chars as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl Borrow<str> for Chars {
    fn borrow(&self) -> &str {
        self
    }
}

impl fmt::Display for Chars {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for Chars {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::Chars;

    #[test]
    fn test_chars_hashmap() {
        let mut table: HashMap<Chars, u64> = HashMap::new();
        table.insert("foo".into(), 5);
        assert_eq!(5, *table.get("foo").unwrap());
    }

    #[test]
    #[cfg_attr(miri, ignore)] // bytes violates SB, see https://github.com/tokio-rs/bytes/issues/522
    fn test_display_and_debug() {
        let s = "test";
        let string: String = s.into();
        let chars: Chars = s.into();

        assert_eq!(format!("{}", string), format!("{}", chars));
        assert_eq!(format!("{:?}", string), format!("{:?}", chars));
    }
}
