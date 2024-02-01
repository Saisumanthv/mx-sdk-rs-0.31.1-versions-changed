use dharitri_wasm::types::heap::H256;
use denali::model::{Step, TransferStep, TxTransfer, ValidatorRewardStep};

use crate::{
    sc_call::tx_dct_transfers_from_denali, tx_execution::execute_sc_call, tx_mock::TxInput,
    world_mock::BlockchainMock,
};

impl BlockchainMock {
    pub fn denali_transfer(&mut self, transfer_step: TransferStep) -> &mut Self {
        self.with_borrowed(|state| ((), execute(state, &transfer_step.tx)));
        self.denali_trace.steps.push(Step::Transfer(transfer_step));
        self
    }

    pub fn denali_validator_reward(
        &mut self,
        validator_rewards_step: ValidatorRewardStep,
    ) -> &mut Self {
        self.increase_validator_reward(
            &validator_rewards_step.tx.to.to_address(),
            &validator_rewards_step.tx.moax_value.value,
        );
        self.denali_trace
            .steps
            .push(Step::ValidatorReward(validator_rewards_step));
        self
    }
}

fn execute(mut state: BlockchainMock, tx_transfer: &TxTransfer) -> BlockchainMock {
    let tx_input = TxInput {
        from: tx_transfer.from.value.clone(),
        to: tx_transfer.to.value.clone(),
        moax_value: tx_transfer.moax_value.value.clone(),
        dct_values: tx_dct_transfers_from_denali(tx_transfer.dct_value.as_slice()),
        func_name: Vec::new(),
        args: Vec::new(),
        gas_limit: tx_transfer.gas_limit.value,
        gas_price: tx_transfer.gas_price.value,
        tx_hash: H256::zero(),
    };

    // nonce gets increased irrespective of whether the tx fails or not
    state.increase_account_nonce(&tx_input.from);

    let (tx_result, state) = execute_sc_call(tx_input, state);
    tx_result.assert_ok();
    state
}
