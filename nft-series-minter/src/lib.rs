#![no_std]

use multiversx_sc::imports::*;
mod nft_module;
pub mod nft_series_minter_proxy;

#[multiversx_sc::contract]
pub trait NftSeriesMinter: nft_module::NftModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(createNft)]
    fn create_nft(
        &self,
        receiver_address: ManagedAddress,
        serials: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mut nfts: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        for serial in serials {
            nfts.push(self.create_nft_with_serial(serial));
        }

        self.send().direct_multi(&receiver_address, &nfts);
    }
}
