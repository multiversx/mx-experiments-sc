multiversx_sc::imports!();

pub type Nonce = u64;

#[multiversx_sc::module]
pub trait SignatureModule {
    #[view(getUserNonce)]
    fn get_user_nonce(&self, user: ManagedAddress) -> Nonce {
        let user_id = self.user_id().get_id(&user);
        self.user_nonce(user_id).get()
    }

    fn check_worker_signature(&self, user: &ManagedAddress, signature: &ManagedBuffer) {
        if cfg!(debug_assertions) {
            return;
        }

        let user_nonce = self.get_and_increment_user_nonce(user);
        let mut encoded_nonce = ManagedBuffer::new();
        let _ = user_nonce.top_encode(&mut encoded_nonce);

        let mut signed_message = user.as_managed_buffer().clone();
        signed_message = signed_message.concat(encoded_nonce);
        self.crypto()
            .verify_ed25519(user.as_managed_buffer(), &signed_message, signature);
    }

    fn get_and_increment_user_nonce(&self, user: &ManagedAddress) -> Nonce {
        let user_id = self.user_id().get_id_or_insert(user);
        self.user_nonce(user_id).update(|user_nonce| {
            let returned_nonce = *user_nonce;
            *user_nonce += 1;

            returned_nonce
        })
    }

    #[storage_mapper("userId")]
    fn user_id(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("userNonce")]
    fn user_nonce(&self, user_id: AddressId) -> SingleValueMapper<Nonce>;

    #[storage_mapper("signer")]
    fn signer(&self) -> SingleValueMapper<ManagedAddress>;
}
