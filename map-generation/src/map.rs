use crate::random::{Random, Seed, U64_BYTES};

multiversx_sc::imports!();

const WIGGLE: i16 = 32;
const TILES: u32 = 512;
const SIZE: u32 = TILES * 2 + 1;
const SQUARED_SIZE: usize = (SIZE * SIZE) as usize;

pub struct Map {
    rng: Random,
    terrain: [u32; SQUARED_SIZE],
}

impl Map {
    #[inline]
    pub fn new(seed: Seed) -> Self {
        Self {
            rng: Random::new(seed),
            terrain: [0; SQUARED_SIZE],
        }
    }

    pub fn new_from_seed(seed: Seed) -> Self {
        Self {
            rng: Random::new(seed),
            terrain: [0; SQUARED_SIZE], // TODO: Generate the terrain
        }
    }

    pub fn get(&self, x: u32, y: u32) -> u32 {
        self.terrain[(x * SIZE + y) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, value: u32) {
        self.terrain[(x * SIZE + y) as usize] = value;
    }

    pub fn init<Api: ManagedTypeApi>(&mut self) {
        let top_left = self.rng.next_u8::<Api>() as u32;
        let bottom_left = self.rng.next_u8::<Api>() as u32;
        let top_right = self.rng.next_u8::<Api>() as u32;
        let bottom_right = self.rng.next_u8::<Api>() as u32;

        self.set(0, 0, top_left);
        self.set(SIZE - 1, 0, bottom_left);
        self.set(0, SIZE - 1, top_right);
        self.set(SIZE - 1, SIZE - 1, bottom_right);
    }

    pub fn square<Api: ManagedTypeApi>(&mut self, x: u32, y: u32, radius: u32) {
        let top_left = self.get(x - radius, y - radius);
        let bottom_left = self.get(x - radius, y + radius);
        let top_right = self.get(x + radius, y - radius);
        let bottom_right = self.get(x + radius, y + radius);
        let average = (top_left + bottom_left + top_right + bottom_right) / 4;
        let height = self.wiggle::<Api>(average, WIGGLE);

        self.set(x, y, height);
    }

    pub fn diamond<Api: ManagedTypeApi>(&mut self, x: u32, y: u32, radius: u32) {
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

        let height = self.wiggle::<Api>(t / spread, WIGGLE);
        self.set(x, y, height);
    }

    pub fn squares<Api: ManagedTypeApi>(&mut self, step: u32) {
        let step2 = step * 2;
        for x in 0..step {
            for y in 0..step {
                let square_x = SIZE / step2 + (x * SIZE / step);
                let square_y = SIZE / step2 + (y * SIZE / step);
                let radius = SIZE / step2;

                self.square::<Api>(square_x, square_y, radius);
            }
        }
    }

    pub fn diamonds<Api: ManagedTypeApi>(&mut self, radius: u32) {
        let radius_usize = radius as usize;
        for x in (0..SIZE).step_by(radius_usize) {
            let y_start: u32 = if (x / (radius)) % 2 == 0 { radius } else { 0 };

            for y in (y_start..SIZE).step_by(radius_usize * 2) {
                self.diamond::<Api>(x, y, radius);
            }
        }
    }

    pub fn wiggle<Api: ManagedTypeApi>(&mut self, value: u32, range: i16) -> u32 {
        let min = if value < range as u32 {
            value as i16
        } else {
            range
        };

        (value as i16 + self.rng.gen_range::<Api>(-min, range)) as u32
    }
}

#[multiversx_sc::module]
pub trait MapModule {
    #[only_owner]
    #[endpoint(generateNewMap)]
    fn generate_new_map(&self) -> [u32; SQUARED_SIZE] {
        let buffer = ManagedBuffer::new_random(U64_BYTES);
        let seed_result = u64::top_decode(buffer);
        require!(seed_result.is_ok(), "Failed decoding random seed");

        let seed = unsafe { seed_result.unwrap_unchecked() };
        let mut map = Map::new(seed);
        let initial_seed = map.rng.seed;

        let steps = TILES.trailing_zeros() + 1;
        for s in 0..steps {
            map.squares::<Self::Api>(1 << s);
            map.diamonds::<Self::Api>(1 << (steps - s - 1));
        }

        // for _x in 0..SIZE {
        //     for _y in 0..SIZE {
        //         // TODO: Do... something... ?
        //     }
        // }

        self.current_map_seed().set(initial_seed);

        map.terrain
    }

    #[view(getCurrentMapSeed)]
    #[storage_mapper("currentMapSeed")]
    fn current_map_seed(&self) -> SingleValueMapper<Seed>;
}
