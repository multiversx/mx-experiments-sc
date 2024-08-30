use multiversx_sc::{derive_imports::*, imports::*};

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
struct NftInfo<M: ManagedTypeApi> {
    name: ManagedBuffer<M>,
    royalties: BigUint<M>,
    hash: ManagedBuffer<M>,
    attributes: ManagedBuffer<M>,
    uri: ManagedVec<M, ManagedBuffer<M>>,
    last_nonce: u64,
}
#[multiversx_sc::module]
pub trait NftModule {
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        royalties: BigUint,
        uri: ManagedVec<ManagedBuffer>,
        attributes: ManagedBuffer,
    ) {
        require!(self.nft_token_id().is_empty(), "Token already issued");
        require!(royalties <= ROYALTIES_MAX, "Royalties cannot exceed 100%");

        self.setup_nft_info(token_name.clone(), royalties, uri, attributes);

        let payment_amount = self.call_value().egld_value();
        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                payment_amount.clone_value(),
                &token_name,
                &token_ticker,
                NonFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_transfer_create_role: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .with_callback(self.callbacks().issue_callback())
            .async_call_and_exit()
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        self.require_token_issued();

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                [EsdtLocalRole::NftCreate][..].iter().cloned(),
            )
            .async_call_and_exit()
    }

    #[callback]
    fn issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<EgldOrEsdtTokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id.unwrap_esdt());
            }
            ManagedAsyncCallResult::Err(_) => {
                let returned = self.call_value().egld_or_single_esdt();
                if returned.token_identifier.is_egld() && returned.amount > 0 {
                    self.tx().to(ToCaller).egld(returned.amount).transfer();
                }
            }
        }
    }

    fn setup_nft_info(
        &self,
        name: ManagedBuffer,
        royalties: BigUint,
        uri: ManagedVec<ManagedBuffer>,
        attributes: ManagedBuffer,
    ) {
        let attributes_sha256 = self.crypto().sha256(&attributes);
        let hash = attributes_sha256.as_managed_buffer();
        self.nft_token_info().set(NftInfo {
            name,
            royalties,
            hash: hash.clone(),
            attributes,
            uri,
            last_nonce: 0u64,
        });
    }

    fn create_nft_with_stored_attributes(
        &self,
        amount_to_mint: u64,
    ) -> ManagedVec<Self::Api, EsdtTokenPayment> {
        self.require_token_issued();
        let nft_info = self.nft_token_info().get();

        let nft_token_id = self.nft_token_id().get();
        let mut nfts: ManagedVec<Self::Api, EsdtTokenPayment> = ManagedVec::new();
        for _ in 0..amount_to_mint {
            let nft_nonce = self.send().esdt_nft_create(
                &nft_token_id,
                &BigUint::from(NFT_AMOUNT),
                &nft_info.name,
                &nft_info.royalties,
                &nft_info.hash,
                &nft_info.attributes,
                &nft_info.uri,
            );
            nfts.push(EsdtTokenPayment::new(
                nft_token_id.clone(),
                nft_nonce,
                1u64.into(),
            ))
        }

        nfts
    }

    fn require_token_issued(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued");
    }

    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("nftTokenInfo")]
    fn nft_token_info(&self) -> SingleValueMapper<NftInfo<Self::Api>>;
}
