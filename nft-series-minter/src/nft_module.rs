use multiversx_sc::{derive_imports::*, imports::*};

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;

static TAGS_PREFIX: &[u8] = b"tags:";
static ATTRIBUTES_SEPARATOR: &[u8] = b";";

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
        require!(self.nft_token().is_empty(), "Token already issued");
        require!(royalties <= ROYALTIES_MAX, "Royalties cannot exceed 100%");

        self.setup_nft_info(token_name.clone(), royalties, uri, attributes);

        let payment_amount = self.call_value().egld_value().clone_value();
        self.nft_token().issue(
            EsdtTokenType::NonFungible,
            payment_amount,
            token_name,
            token_ticker,
            0,
            Some(self.callbacks().issue_callback()),
        );
    }

    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        self.require_token_issued();

        self.nft_token()
            .set_local_roles(&[EsdtLocalRole::NftCreate], None);
    }

    #[callback]
    fn issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<EgldOrEsdtTokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token().set_token_id(token_id.unwrap_esdt());
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

    fn create_nft_with_serial(&self, serial: ManagedBuffer) -> EsdtTokenPayment {
        self.require_token_issued();

        let nft_info = self.nft_token_info().get();
        let tags_attributes = self.build_attributes_tags_part(&serial);
        let mut attributes = nft_info.attributes;
        attributes.append_bytes(ATTRIBUTES_SEPARATOR);
        attributes.append(&tags_attributes);

        let nft_token_id = self.nft_token().get_token_id();
        let nft_nonce = self.send().esdt_nft_create(
            &nft_token_id,
            &BigUint::from(NFT_AMOUNT),
            &nft_info.name,
            &nft_info.royalties,
            &nft_info.hash,
            &attributes,
            &nft_info.uri,
        );
        EsdtTokenPayment::new(nft_token_id.clone(), nft_nonce, 1u64.into())
    }

    fn build_attributes_tags_part(
        &self,
        serial: &ManagedBuffer<Self::Api>,
    ) -> ManagedBuffer<Self::Api> {
        let mut tags_attributes = ManagedBuffer::new_from_bytes(TAGS_PREFIX);
        tags_attributes.append(serial);

        tags_attributes
    }

    fn require_token_issued(&self) {
        require!(!self.nft_token().is_empty(), "Token not issued");
    }

    #[storage_mapper("nftTokenId")]
    fn nft_token(&self) -> NonFungibleTokenMapper;

    #[storage_mapper("nftTokenInfo")]
    fn nft_token_info(&self) -> SingleValueMapper<NftInfo<Self::Api>>;
}
