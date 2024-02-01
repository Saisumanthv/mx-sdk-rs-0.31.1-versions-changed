use crate::{
    num_bigint,
    world_mock::{is_smart_contract_address, DctData, DctInstance},
    DebugApi,
};
use dharitri_wasm::{
    api::{BlockchainApi, BlockchainApiImpl, Handle, ManagedTypeApi},
    types::{
        heap::{Address, H256},
        BigUint, DctLocalRole, DctLocalRoleFlags, DctTokenData, DctTokenType, ManagedAddress,
        ManagedBuffer, ManagedType, ManagedVec, TokenIdentifier,
    },
};

impl BlockchainApi for DebugApi {
    type BlockchainApiImpl = DebugApi;

    fn blockchain_api_impl() -> Self::BlockchainApiImpl {
        DebugApi::new_from_static()
    }
}

impl BlockchainApiImpl for DebugApi {
    fn get_caller_legacy(&self) -> Address {
        self.input_ref().from.clone()
    }

    fn get_sc_address_legacy(&self) -> Address {
        self.input_ref().to.clone()
    }

    fn get_owner_address_legacy(&self) -> Address {
        self.with_contract_account(|account| {
            account
                .contract_owner
                .clone()
                .unwrap_or_else(|| panic!("contract owner address not set"))
        })
    }

    fn get_shard_of_address_legacy(&self, _address: &Address) -> u32 {
        panic!("get_shard_of_address not implemented")
    }

    fn is_smart_contract_legacy(&self, address: &Address) -> bool {
        is_smart_contract_address(address)
    }

    fn load_balance_legacy(&self, dest: Handle, address: &Address) {
        assert!(
            address == &self.get_sc_address_legacy(),
            "get balance not yet implemented for accounts other than the contract itself"
        );
        let moax_balance = self.with_contract_account(|account| account.moax_balance.clone());
        self.bi_overwrite(dest, moax_balance.into());
    }

    fn get_state_root_hash_legacy(&self) -> H256 {
        panic!("get_state_root_hash_legacy not yet implemented")
    }

    fn get_tx_hash_legacy(&self) -> H256 {
        self.input_ref().tx_hash.clone()
    }

    fn get_gas_left(&self) -> u64 {
        self.input_ref().gas_limit
    }

    fn get_block_timestamp(&self) -> u64 {
        self.blockchain_ref().current_block_info.block_timestamp
    }

    fn get_block_nonce(&self) -> u64 {
        self.blockchain_ref().current_block_info.block_nonce
    }

    fn get_block_round(&self) -> u64 {
        self.blockchain_ref().current_block_info.block_round
    }

    fn get_block_epoch(&self) -> u64 {
        self.blockchain_ref().current_block_info.block_epoch
    }

    fn get_block_random_seed_legacy(&self) -> Box<[u8; 48]> {
        self.blockchain_ref()
            .current_block_info
            .block_random_seed
            .clone()
    }

    fn get_prev_block_timestamp(&self) -> u64 {
        self.blockchain_ref().previous_block_info.block_timestamp
    }

    fn get_prev_block_nonce(&self) -> u64 {
        self.blockchain_ref().previous_block_info.block_nonce
    }

    fn get_prev_block_round(&self) -> u64 {
        self.blockchain_ref().previous_block_info.block_round
    }

    fn get_prev_block_epoch(&self) -> u64 {
        self.blockchain_ref().previous_block_info.block_epoch
    }

    fn get_prev_block_random_seed_legacy(&self) -> Box<[u8; 48]> {
        self.blockchain_ref()
            .previous_block_info
            .block_random_seed
            .clone()
    }

    fn get_current_dct_nft_nonce(&self, address_handle: Handle, token_id_handle: Handle) -> u64 {
        let address = ManagedAddress::<DebugApi>::from_raw_handle(address_handle);
        assert!(
            address.to_address() == self.get_sc_address_legacy(),
            "get_current_dct_nft_nonce not yet implemented for accounts other than the contract itself"
        );

        self.with_contract_account(|account| {
            account
                .dct
                .get_by_identifier_or_default(
                    TokenIdentifier::<DebugApi>::from_raw_handle(token_id_handle)
                        .to_dct_identifier()
                        .as_slice(),
                )
                .last_nonce
        })
    }

    fn load_dct_balance(
        &self,
        address_handle: Handle,
        token_id_handle: Handle,
        nonce: u64,
        dest: Handle,
    ) {
        let address = ManagedAddress::<DebugApi>::from_raw_handle(address_handle);
        assert!(
            address.to_address() == self.get_sc_address_legacy(),
            "get_dct_balance not yet implemented for accounts other than the contract itself"
        );

        let dct_balance = self.with_contract_account(|account| {
            account.dct.get_dct_balance(
                TokenIdentifier::<DebugApi>::from_raw_handle(token_id_handle)
                    .to_dct_identifier()
                    .as_slice(),
                nonce,
            )
        });
        self.bi_overwrite(dest, dct_balance.into());
    }

    fn get_dct_token_data<M: ManagedTypeApi>(
        &self,
        address: &ManagedAddress<M>,
        token: &TokenIdentifier<M>,
        nonce: u64,
    ) -> DctTokenData<M> {
        self.blockchain_cache()
            .with_account(&address.to_address(), |account| {
                let token_identifier_value = token.to_dct_identifier();
                if let Some(dct_data) = account
                    .dct
                    .get_by_identifier(token_identifier_value.as_slice())
                {
                    if let Some(instance) = dct_data.instances.get_by_nonce(nonce) {
                        self.dct_token_data_from_instance(
                            token_identifier_value.into_vec(),
                            dct_data,
                            nonce,
                            instance,
                        )
                    } else {
                        // missing nonce
                        DctTokenData {
                            token_type: DctTokenType::based_on_token_nonce(nonce),
                            ..Default::default()
                        }
                    }
                } else {
                    // missing token identifier
                    DctTokenData {
                        token_type: DctTokenType::Fungible,
                        ..Default::default()
                    }
                }
            })
    }

    fn get_dct_token_data_unmanaged<M: ManagedTypeApi>(
        &self,
        _address: &ManagedAddress<M>,
        _token: &TokenIdentifier<M>,
        _nonce: u64,
    ) -> DctTokenData<M> {
        panic!("get_dct_token_data_unmanaged is deprecated and should never be used in Rust tests")
    }

    fn get_dct_local_roles(&self, token_id_handle: Handle) -> DctLocalRoleFlags {
        let sc_address = self.input_ref().to.clone();
        self.blockchain_cache()
            .with_account(&sc_address, |account| {
                let mut result = DctLocalRoleFlags::NONE;
                if let Some(dct_data) = account.dct.get_by_identifier(
                    TokenIdentifier::<DebugApi>::from_raw_handle(token_id_handle)
                        .to_dct_identifier()
                        .as_slice(),
                ) {
                    for role_name in dct_data.roles.get() {
                        result |= DctLocalRole::from(role_name.as_slice()).to_flag();
                    }
                }

                result
            })
    }
}

impl DebugApi {
    fn dct_token_data_from_instance<M: ManagedTypeApi>(
        &self,
        token_identifier_value: Vec<u8>,
        dct_data: &DctData,
        nonce: u64,
        instance: &DctInstance,
    ) -> DctTokenData<M> {
        let creator = if let Some(creator) = &instance.metadata.creator {
            ManagedAddress::from_address(creator)
        } else {
            ManagedAddress::zero()
        };

        let mut uris = ManagedVec::new();
        for uri in &instance.metadata.uri {
            uris.push(ManagedBuffer::new_from_bytes(uri.as_slice()));
        }

        DctTokenData {
            token_type: DctTokenType::based_on_token_nonce(nonce),
            amount: BigUint::from_raw_handle(self.insert_new_big_uint(instance.balance.clone())),
            frozen: dct_data.frozen,
            hash: ManagedBuffer::from_raw_handle(
                self.insert_new_managed_buffer(instance.metadata.hash.clone().unwrap_or_default()),
            ),
            name: ManagedBuffer::from_raw_handle(
                self.insert_new_managed_buffer(token_identifier_value),
            ),
            attributes: ManagedBuffer::from_raw_handle(
                self.insert_new_managed_buffer(instance.metadata.attributes.clone()),
            ),
            creator,
            royalties: BigUint::from_raw_handle(
                self.insert_new_big_uint(num_bigint::BigUint::from(instance.metadata.royalties)),
            ),
            uris,
        }
    }
}
