#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait OwnerSc {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(setGasBurnerAddress)]
    fn set_gas_burner_address(&self, gas_burner: ManagedAddress) {
        require!(
            !gas_burner.is_zero() && self.blockchain().is_smart_contract(&gas_burner),
            "Invalid SC address"
        );

        self.gas_burner().set(gas_burner);
    }

    #[endpoint(claimDevRewards)]
    fn claim_dev_rewards(&self) -> BigUint {
        let caller = self.blockchain().get_caller();
        let gas_burner = self.gas_burner().get();
        require!(
            caller == gas_burner,
            "Only the gas burner SC may call this endpoint"
        );

        self.send().claim_developer_rewards(gas_burner).sync_call();

        let own_sc_address = self.blockchain().get_sc_address();
        let egld_balance_after = self.blockchain().get_balance(&own_sc_address);
        self.send()
            .direct_non_zero_egld(&caller, &egld_balance_after);

        egld_balance_after
    }

    #[storage_mapper("gasBurner")]
    fn gas_burner(&self) -> SingleValueMapper<ManagedAddress>;
}
