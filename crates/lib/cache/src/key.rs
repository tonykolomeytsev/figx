use std::hash::Hasher;

use bincode::{BorrowDecode, Decode, Encode};
use bytes::Bytes;
use xxhash_rust::xxh64::Xxh64;

#[derive(Clone, Hash, Eq, PartialEq)]
#[non_exhaustive]
pub struct CacheKey {
    hash: Bytes,
}

impl Encode for CacheKey {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.hash.as_ref().encode(encoder)
    }
}

impl<Context> Decode<Context> for CacheKey {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let bytes: Vec<u8> = Decode::decode(decoder)?;
        Ok(Self {
            hash: Bytes::from(bytes),
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for CacheKey {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let bytes: &'de [u8] = BorrowDecode::borrow_decode(decoder)?;
        Ok(Self {
            hash: Bytes::copy_from_slice(bytes),
        })
    }
}

impl CacheKey {
    pub fn builder() -> CacheKeyBuilder {
        CacheKeyBuilder {
            hasher: Xxh64::new(42),
            tag: 0,
        }
    }

    pub fn tag(&self) -> u8 {
        let buf: &[u8] = self.as_ref();
        *buf.first().expect("cache key has never empty buffer")
    }
}

impl AsRef<[u8]> for CacheKey {
    fn as_ref(&self) -> &[u8] {
        &self.hash
    }
}

impl std::fmt::Debug for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.hash.iter() {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct CacheKeyBuilder {
    hasher: xxhash_rust::xxh64::Xxh64,
    tag: u8,
}

macro_rules! write_impl {
    ($name:ident, $t:ty) => {
        pub fn $name(mut self, value: $t) -> Self {
            self.hasher.$name(value);
            self
        }
    };
}

impl CacheKeyBuilder {
    write_impl!(write, &[u8]);
    write_impl!(write_u8, u8);
    write_impl!(write_u16, u16);
    write_impl!(write_u32, u32);
    write_impl!(write_u64, u64);
    write_impl!(write_u128, u128);
    write_impl!(write_usize, usize);
    write_impl!(write_i8, i8);
    write_impl!(write_i16, i16);
    write_impl!(write_i32, i32);
    write_impl!(write_i64, i64);
    write_impl!(write_i128, i128);
    write_impl!(write_isize, isize);

    pub fn write_str(mut self, s: &str) -> Self {
        self.hasher.write(s.as_bytes());
        self
    }

    pub fn write_bool(mut self, b: bool) -> Self {
        self.hasher.write_u8(if b { 1 } else { 2 });
        self
    }

    pub fn set_tag(mut self, tag: u8) -> Self {
        self.tag = tag;
        self
    }

    pub fn build(self) -> CacheKey {
        let mut buf = [0u8; 9];
        buf[0] = self.tag;
        buf[1..].copy_from_slice(&self.hasher.digest().to_be_bytes());
        CacheKey {
            hash: Bytes::from_owner(buf),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use crate::CacheKey;

    #[test]
    fn same_operators_order__LEADS_TO__same_keys() {
        let key1 = CacheKey::builder()
            .write(b"123")
            .write_bool(true)
            .write_i128(1)
            .write_u128(2)
            .write_str("hello")
            .build();
        let key2 = CacheKey::builder()
            .write(b"123")
            .write_bool(true)
            .write_i128(1)
            .write_u128(2)
            .write_str("hello")
            .build();
        assert_eq!(key1, key2);
    }

    #[test]
    fn different_operators_order__LEADS_TO__different_keys() {
        let key1 = CacheKey::builder()
            .write(b"123")
            .write_bool(true)
            .write_i128(1)
            .write_u128(2)
            .write_str("hello")
            .build();
        let key2 = CacheKey::builder()
            .write(b"123")
            .write_i128(1)
            .write_bool(true)
            .write_u128(2)
            .write_str("hello")
            .build();
        assert_ne!(key1, key2);
    }

    #[test]
    fn tag_is_always_inside_key() {
        let key = CacheKey::builder().set_tag(42).write_str("hello").build();
        assert_eq!(42, key.hash[0]);
    }

    #[test]
    fn test_ser_de() {
        let source_key = CacheKey::builder()
            .set_tag(123)
            .write_bool(true)
            .write_i8(1)
            .build();
        let serialized_key =
            bincode::encode_to_vec(&source_key, bincode::config::standard()).unwrap();
        let (deserialized_key, _): (CacheKey, usize) =
            bincode::decode_from_slice(&serialized_key, bincode::config::standard()).unwrap();
        assert_eq!(source_key, deserialized_key);
    }
}
