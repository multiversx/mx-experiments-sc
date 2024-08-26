use multiversx_sc::api::CryptoApi;

use crate::random::{Hash, Random};

multiversx_sc::imports!();

const WIGGLE: i16 = 32;
const TILES: u32 = 512;
const SIZE: u32 = TILES * 2 + 1;
const SQUARED_SIZE: usize = (SIZE * SIZE) as usize;

pub struct Map<M: ManagedTypeApi + CryptoApi> {
    rng: Random<M>,
    terrain: [u32; SQUARED_SIZE],
}

impl<M: ManagedTypeApi + CryptoApi> Default for Map<M> {
    #[inline]
    fn default() -> Self {
        Self {
            rng: Random::new(),
            terrain: [0; SQUARED_SIZE],
        }
    }
}

impl<M: ManagedTypeApi + CryptoApi> Map<M> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from_seed(seed: Hash<M>) -> Self {
        Self {
            rng: Random::from_hash(seed, 0),
            terrain: [0; SQUARED_SIZE], // TODO: Generate the terrain
        }
    }

    pub fn get(&self, x: u32, y: u32) -> u32 {
        self.terrain[(x * SIZE + y) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, value: u32) {
        self.terrain[(x * SIZE + y) as usize] = value;
    }

    pub fn init(&mut self) {
        let top_left = self.rng.next_u8() as u32;
        let bottom_left = self.rng.next_u8() as u32;
        let top_right = self.rng.next_u8() as u32;
        let bottom_right = self.rng.next_u8() as u32;

        self.set(0, 0, top_left);
        self.set(SIZE - 1, 0, bottom_left);
        self.set(0, SIZE - 1, top_right);
        self.set(SIZE - 1, SIZE - 1, bottom_right);
    }

    pub fn square(&mut self, x: u32, y: u32, radius: u32) {
        let top_left = self.get(x - radius, y - radius);
        let bottom_left = self.get(x - radius, y + radius);
        let top_right = self.get(x + radius, y - radius);
        let bottom_right = self.get(x + radius, y + radius);
        let average = (top_left + bottom_left + top_right + bottom_right) / 4;
        let height = self.wiggle(average, WIGGLE);

        self.set(x, y, height);
    }

    pub fn diamond(&mut self, x: u32, y: u32, radius: u32) {
        let mut spread = 0;
        let mut t = 0;

        if radius <= x {
            spread += 1;
            t += self.get(x - radius, y);
        }

        if x + radius < SIZE {
            spread += 1;
            t += self.get(x + radius, y);
        }

        if radius <= y {
            spread += 1;
            t += self.get(x, y - radius);
        }

        if y + radius < SIZE {
            spread += 1;
            t += self.get(x, y + radius);
        }

        let height = self.wiggle(t / spread, WIGGLE);
        self.set(x, y, height);
    }

    pub fn squares(&mut self, step: u32) {
        let step2 = step * 2;
        for x in 0..step {
            for y in 0..step {
                let square_x = SIZE / step2 + (x * SIZE / step);
                let square_y = SIZE / step2 + (y * SIZE / step);
                let radius = SIZE / step2;

                self.square(square_x, square_y, radius);
            }
        }
    }

    pub fn diamonds(&mut self, radius: u32) {
        let radius_usize = radius as usize;
        for x in (0..SIZE).step_by(radius_usize) {
            let y_start: u32 = if (x / (radius)) % 2 == 0 { radius } else { 0 };

            for y in (y_start..SIZE).step_by(radius_usize * 2) {
                self.diamond(x, y, radius);
            }
        }
    }

    pub fn wiggle(&mut self, value: u32, range: i16) -> u32 {
        let min = if value < range as u32 {
            value as i16
        } else {
            range
        };

        (value as i16 + self.rng.gen_range(-min, range)) as u32
    }
}

#[multiversx_sc::module]
pub trait MapModule {
    #[only_owner]
    #[endpoint(generateNewMap)]
    fn generate_new_map(&self) -> Hash<Self::Api> {
        let mut map = Map::<Self::Api>::new();
        let initial_seed = Hash::from_raw_handle(map.rng.seed.get_raw_handle()).clone();

        let steps = TILES.trailing_zeros() + 1;
        for s in 0..steps {
            map.squares(1 << s);
            map.diamonds(1 << (steps - s - 1));
        }

        for _x in 0..SIZE {
            for _y in 0..SIZE {
                // TODO: Do... something... ?
            }
        }

        self.current_map_seed().set(&initial_seed);

        initial_seed
    }

    #[view(getCurrentMapSeed)]
    #[storage_mapper("currentMapSeed")]
    fn current_map_seed(&self) -> SingleValueMapper<Hash<Self::Api>>;
}
