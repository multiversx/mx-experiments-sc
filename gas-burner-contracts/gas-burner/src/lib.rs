#![no_std]

multiversx_sc::imports!();

pub mod leaderboard;
pub mod rewards;
pub mod signature;
pub mod week_timekeeping;
pub mod work;

#[multiversx_sc::contract]
pub trait GasBurner:
    work::WorkModule
    + leaderboard::LeaderboardModule
    + rewards::RewardsModule
    + signature::SignatureModule
    + week_timekeeping::WeekTimekeepingModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self, owner_sc: ManagedAddress, signer: ManagedAddress, token_id: TokenIdentifier) {
        require!(token_id.is_valid_esdt_identifier(), "Invalid ESDT token");
        require!(
            !owner_sc.is_zero() && self.blockchain().is_smart_contract(&owner_sc),
            "Invalid SC address"
        );

        self.owner_sc().set(owner_sc);
        self.signer().set(signer);
        self.token().set_token_id(token_id);

        let current_epoch = self.blockchain().get_block_epoch();
        self.first_week_start_epoch().set(current_epoch);

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
