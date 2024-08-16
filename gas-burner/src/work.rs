use crate::week_timekeeping::Week;

multiversx_sc::imports!();

const DIV_CONST: u64 = 1_000_000;
const GAS_TO_FINISH_ENDPOINT: u64 = 1_000;

#[multiversx_sc::module]
pub trait WorkModule:
    crate::signature::SignatureModule
    + crate::leaderboard::LeaderboardModule
    + crate::week_timekeeping::WeekTimekeepingModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// Signature uses ed25519 and you must sign the message of user_address + nonce
    /// Nonce starts from 0 and you can get it through the getUserNonce view
    #[endpoint]
    fn work(&self, signature: ManagedBuffer) -> EsdtTokenPayment {
        self.require_not_paused();

        let gas_left = self.blockchain().get_gas_left();
        let tokens_to_send = gas_left / DIV_CONST;
        require!(tokens_to_send > 0, "Gas too low");

        let caller = self.blockchain().get_caller();
        let minted_tokens = self.token().mint_and_send(&caller, tokens_to_send.into());

        self.check_worker_signature(&caller, &signature);
        self.increase_leaderboard_entry(&caller, gas_left);
        self.use_remaining_gas();

        minted_tokens
    }

    fn use_remaining_gas(&self) {
        let mut gas_left = self.blockchain().get_gas_left();
        while gas_left > GAS_TO_FINISH_ENDPOINT {
            gas_left = self.blockchain().get_gas_left();
        }
    }

    #[view(getTokenId)]
    #[storage_mapper("tokenId")]
    fn token(&self) -> FungibleTokenMapper;

    #[storage_mapper("workersForWeek")]
    fn workers_for_week(&self, week: Week) -> UnorderedSetMapper<AddressId>;

    #[storage_mapper("userWorkForWeek")]
    fn user_work_for_week(&self, user_id: AddressId) -> SingleValueMapper<BigUint>;
}
