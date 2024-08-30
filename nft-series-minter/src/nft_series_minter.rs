#![no_std]

use multiversx_sc::imports::*;
mod nft_module;

#[multiversx_sc::contract]
pub trait NftSeriesMinter: nft_module::NftModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(createNft)]
    fn create_nft(&self, amount_to_mint: u64, receiver_address: ManagedAddress) {
        let nfts = self.create_nft_with_stored_attributes(amount_to_mint);
        self.send().direct_multi(&receiver_address, &nfts);
    }
}
