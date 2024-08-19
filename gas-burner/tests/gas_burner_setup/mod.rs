use gas_burner::{work::WorkModule, GasBurner};
use multiversx_sc::types::{Address, EsdtLocalRole};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, ContractObjWrapper},
    managed_address, managed_buffer, managed_token_id, rust_biguint, DebugApi,
};

pub static GAS_BURNER_TOKEN_ID: &[u8] = b"GAS-123456";

pub struct GasBurnerSetup<GasBurnerBuilder>
where
    GasBurnerBuilder: 'static + Copy + Fn() -> gas_burner::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub signer: Address,
    pub first_user: Address,
    pub second_user: Address,
    pub gas_burner_owner: Address,
    pub gas_burner_wrapper: ContractObjWrapper<gas_burner::ContractObj<DebugApi>, GasBurnerBuilder>,
}

impl<GasBurnerBuilder> GasBurnerSetup<GasBurnerBuilder>
where
    GasBurnerBuilder: 'static + Copy + Fn() -> gas_burner::ContractObj<DebugApi>,
{
    pub fn new(gas_burner_builder: GasBurnerBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let signer = b_mock.create_user_account(&rust_zero);
        let first_user = b_mock.create_user_account(&rust_zero);
        let second_user = b_mock.create_user_account(&rust_zero);
        let gas_burner_owner = b_mock.create_user_account(&rust_zero);
        let gas_burner_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&gas_burner_owner),
            gas_burner_builder,
            "gas burner",
        );

        b_mock
            .execute_tx(&gas_burner_owner, &gas_burner_wrapper, &rust_zero, |sc| {
                sc.init(
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
            gas_burner_owner,
            gas_burner_wrapper,
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
