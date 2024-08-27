multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const U8_BYTES: usize = 1;
const I16_BYTES: usize = 2;
pub const U64_BYTES: usize = 8;
pub const HASH_LEN: usize = 32;
static FAILED_DECODE_ERR_MSG: &[u8] = b"Failed decoding u64";

pub type Seed = u64;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Random {
    pub seed: Seed,
    pub index: usize,
}

impl Random {
    #[inline]
    pub fn new(rand_seed: Seed) -> Self {
        Self {
            seed: rand_seed,
            index: 0,
        }
    }

    #[inline]
    pub fn next_u8<Api: ManagedTypeApi>(&mut self) -> u8 {
        let value = self.next_value::<Api>(U8_BYTES);

        value as u8
    }

    #[inline]
    pub fn next_i16<Api: ManagedTypeApi>(&mut self) -> i16 {
        let value = self.next_value::<Api>(I16_BYTES);

        value as i16
    }

    pub fn gen_range<Api: ManagedTypeApi>(&mut self, min: i16, max: i16) -> i16 {
        let rand = self.next_i16::<Api>();

        if min >= max {
            min
        } else {
            min + rand % (max - min)
        }
    }

    fn next_value<Api: ManagedTypeApi>(&mut self, size: usize) -> u64 {
        if self.index + size > U64_BYTES {
            self.xor_shift_seed();
        }

        let seed_bytes = self.seed.to_be_bytes();
        let val_range = &seed_bytes[self.index..self.index + size];
        let decode_result = u64::top_decode(val_range);
        if decode_result.is_err() {
            Api::error_api_impl().signal_error(FAILED_DECODE_ERR_MSG);
        }

        let rand = unsafe { decode_result.unwrap_unchecked() };
        self.index += size;

        rand
    }

    // Stolen from: https://en.wikipedia.org/wiki/Xorshift
    fn xor_shift_seed(&mut self) {
        let mut new_seed = self.seed;
        new_seed ^= new_seed << 13;
        new_seed ^= new_seed >> 7;
        new_seed ^= new_seed << 17;
        self.seed = new_seed;
        self.index = 0;
    }
}

#[cfg(test)]
mod randomness_tests {
    use multiversx_sc_scenario::DebugApi;

    use super::*;

    #[test]
    fn gen_range_test() {
        let mut random = Random::new(0);
        let rand_nr = random.gen_range::<DebugApi>(-20, -10);
        assert_eq!(rand_nr, -20);

        let rand_nr = random.gen_range::<DebugApi>(10, 20);
        assert_eq!(rand_nr, 10);

        let rand_nr = random.gen_range::<DebugApi>(-20, 20);
        assert_eq!(rand_nr, -20);
    }
}
