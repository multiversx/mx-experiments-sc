#![no_std]

multiversx_sc::imports!();

pub mod map;
pub mod random;

#[multiversx_sc::contract]
pub trait MapGeneration: map::MapModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
