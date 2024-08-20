use gas_burner::{work::WorkModule, GasBurner};
use multiversx_sc::types::{Address, EsdtLocalRole};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, ContractObjWrapper},
    managed_address, managed_buffer, managed_token_id, rust_biguint, DebugApi,
};
use owner_sc::OwnerSc;

pub static GAS_BURNER_TOKEN_ID: &[u8] = b"GAS-123456";

pub struct GasBurnerSetup<GasBurnerBuilder, OwnerScBuilder>
where
    GasBurnerBuilder: 'static + Copy + Fn() -> gas_burner::ContractObj<DebugApi>,
    OwnerScBuilder: 'static + Copy + Fn() -> owner_sc::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub signer: Address,
    pub first_user: Address,
    pub second_user: Address,
    pub owner: Address,
    pub gas_burner_wrapper: ContractObjWrapper<gas_burner::ContractObj<DebugApi>, GasBurnerBuilder>,
    pub owner_sc_wrapper: ContractObjWrapper<owner_sc::ContractObj<DebugApi>, OwnerScBuilder>,
}

impl<GasBurnerBuilder, OwnerScBuilder> GasBurnerSetup<GasBurnerBuilder, OwnerScBuilder>
where
    GasBurnerBuilder: 'static + Copy + Fn() -> gas_burner::ContractObj<DebugApi>,
    OwnerScBuilder: 'static + Copy + Fn() -> owner_sc::ContractObj<DebugApi>,
{
    pub fn new(gas_burner_builder: GasBurnerBuilder, owner_sc_builder: OwnerScBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let signer = b_mock.create_user_account(&rust_zero);
        let first_user = b_mock.create_user_account(&rust_zero);
        let second_user = b_mock.create_user_account(&rust_zero);
        let owner = b_mock.create_user_account(&rust_zero);
        let owner_sc_wrapper =
            b_mock.create_sc_account(&rust_zero, Some(&owner), owner_sc_builder, "owner SC");
        let gas_burner_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(owner_sc_wrapper.address_ref()),
            gas_burner_builder,
            "gas burner",
        );

        b_mock
            .execute_tx(&owner, &owner_sc_wrapper, &rust_zero, |sc| {
                sc.init();

                sc.gas_burner()
                    .set(managed_address!(gas_burner_wrapper.address_ref()));
            })
            .assert_ok();

        b_mock
            .execute_tx(&owner, &gas_burner_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_address!(owner_sc_wrapper.address_ref()),
                    managed_address!(&signer),
                    managed_token_id!(GAS_BURNER_TOKEN_ID),
                );

                sc.paused_status().set(false);
            })
            .assert_ok();

        b_mock.set_esdt_local_roles(
            gas_burner_wrapper.address_ref(),
            GAS_BURNER_TOKEN_ID,
            &[EsdtLocalRole::Mint],
        );

        Self {
            b_mock,
            signer,
            first_user,
            second_user,
            owner,
            gas_burner_wrapper,
            owner_sc_wrapper,
        }
    }

    pub fn work_user(&mut self, user: &Address) {
        self.b_mock
            .execute_tx(user, &self.gas_burner_wrapper, &rust_biguint!(0), |sc| {
                sc.work(managed_buffer!(b"signature"));
            })
            .assert_ok();
    }
}
