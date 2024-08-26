// Stolen (and adapted) from launchpad: https://github.com/multiversx/mx-launchpad-sc/blob/main/launchpad-common/src/random.rs

use multiversx_sc::api::{CryptoApi, CryptoApiImpl};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const U8_BYTES: usize = 1;
const I16_BYTES: usize = 2;
pub const HASH_LEN: usize = 32;
static FAILED_COPY_ERR_MSG: &[u8] = b"Failed copy to/from managed buffer";

pub type Hash<M> = ManagedByteArray<M, HASH_LEN>;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Random<M: ManagedTypeApi + CryptoApi> {
    pub seed: ManagedBuffer<M>,
    pub index: usize,
}

impl<M: ManagedTypeApi + CryptoApi> Default for Random<M> {
    fn default() -> Self {
        Self {
            seed: ManagedBuffer::new_random(HASH_LEN),
            index: 0,
        }
    }
}

impl<M: ManagedTypeApi + CryptoApi> Random<M> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_hash(hash: Hash<M>, index: usize) -> Self {
        Self {
            seed: ManagedBuffer::from_raw_handle(hash.get_raw_handle()),
            index,
        }
    }

    #[inline]
    pub fn next_u8(&mut self) -> u8 {
        let value = self.next_value(U8_BYTES);

        value as u8
    }

    #[inline]
    pub fn next_i16(&mut self) -> i16 {
        let value = self.next_value(I16_BYTES);

        value as i16
    }

    // TODO: Test if this actually works properly with negative numbers
    pub fn gen_range(&mut self, min: i16, max: i16) -> i16 {
        let rand = self.next_i16();

        if min >= max {
            min
        } else {
            min + rand % (max - min)
        }
    }

    fn next_value(&mut self, size: usize) -> u64 {
        if self.index + size > HASH_LEN {
            self.hash_seed();
        }

        let raw_buffer = match self.seed.copy_slice(self.index, size) {
            Some(buffer) => buffer,
            None => M::error_api_impl().signal_error(FAILED_COPY_ERR_MSG),
        };
        let rand = u64::top_decode(raw_buffer).unwrap_or_default();

        self.index += size;

        rand
    }

    fn hash_seed(&mut self) {
        let handle = self.seed.get_raw_handle();
        M::crypto_api_impl().sha256_managed(handle.into(), handle.into());

        self.index = 0;
    }
}
