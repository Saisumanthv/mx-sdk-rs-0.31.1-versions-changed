#![no_std]
#![allow(clippy::type_complexity)]

use dharitri_wasm::dharitri_codec::Empty;

dharitri_wasm::imports!();

/// General test contract.
/// Used especially for investigating async calls and contract interaction in general.
#[dharitri_wasm::contract]
pub trait Vault {
    #[init]
    fn init(&self, opt_arg_to_echo: OptionalValue<ManagedBuffer>) -> OptionalValue<ManagedBuffer> {
        opt_arg_to_echo
    }

    #[endpoint]
    fn echo_arguments(
        &self,
        args: MultiValueEncoded<ManagedBuffer>,
    ) -> MultiValueEncoded<ManagedBuffer> {
        self.call_counts(ManagedBuffer::from(b"echo_arguments"))
            .update(|c| *c += 1);
        args
    }

    #[endpoint]
    fn echo_caller(&self) -> ManagedAddress {
        self.blockchain().get_caller()
    }

    fn dct_transfers_multi(&self) -> MultiValueEncoded<DctTokenPaymentMultiValue> {
        let dct_transfers = self.call_value().all_dct_transfers();
        let mut dct_transfers_multi = MultiValueEncoded::new();
        for dct_transfer in dct_transfers.into_iter() {
            dct_transfers_multi.push(dct_transfer.into_multi_value());
        }
        dct_transfers_multi
    }

    #[payable("*")]
    #[endpoint]
    fn accept_funds(&self) {
        let dct_transfers_multi = self.dct_transfers_multi();
        self.accept_funds_event(&self.call_value().moax_value(), &dct_transfers_multi);

        self.call_counts(ManagedBuffer::from(b"accept_funds"))
            .update(|c| *c += 1);
    }

    #[payable("*")]
    #[endpoint]
    fn accept_funds_echo_payment(
        &self,
    ) -> MultiValue2<BigUint, MultiValueEncoded<DctTokenPaymentMultiValue>> {
        let moax_value = self.call_value().moax_value();
        let dct_transfers_multi = self.dct_transfers_multi();
        self.accept_funds_event(&moax_value, &dct_transfers_multi);

        self.call_counts(ManagedBuffer::from(b"accept_funds_echo_payment"))
            .update(|c| *c += 1);

        (moax_value, dct_transfers_multi).into()
    }

    #[payable("*")]
    #[endpoint]
    fn reject_funds(&self) {
        let dct_transfers_multi = self.dct_transfers_multi();
        self.reject_funds_event(&self.call_value().moax_value(), &dct_transfers_multi);
        sc_panic!("reject_funds");
    }

    #[payable("*")]
    #[endpoint]
    fn retrieve_funds_with_transfer_exec(
        &self,
        #[payment_multi] _payments: ManagedVec<DctTokenPayment<Self::Api>>,
        token: TokenIdentifier,
        amount: BigUint,
        opt_receive_func: OptionalValue<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();
        let func_name = opt_receive_func.into_option().unwrap_or_default();

        Self::Api::send_api_impl()
            .direct_dct_execute(
                &caller,
                &token,
                &amount,
                50_000_000,
                &func_name,
                &ManagedArgBuffer::new_empty(),
            )
            .unwrap_or_else(|_| sc_panic!("DCT transfer failed"));
    }

    #[endpoint]
    fn retrieve_funds(
        &self,
        token: TokenIdentifier,
        nonce: u64,
        amount: BigUint,
        return_message: OptionalValue<ManagedBuffer>,
    ) {
        self.retrieve_funds_event(&token, nonce, &amount);

        let caller = self.blockchain().get_caller();
        let data = match return_message {
            OptionalValue::Some(data) => data,
            OptionalValue::None => ManagedBuffer::new(),
        };

        if token.is_moax() {
            self.send().direct_moax(&caller, &amount, data);
        } else {
            self.send()
                .transfer_dct_via_async_call(&caller, &token, nonce, &amount, data);
        }
    }

    #[endpoint]
    fn retrieve_multi_funds_async(
        &self,
        token_payments: MultiValueEncoded<MultiValue3<TokenIdentifier, u64, BigUint>>,
    ) {
        let caller = self.blockchain().get_caller();
        let mut all_payments = ManagedVec::new();

        for multi_arg in token_payments.into_iter() {
            let (token_id, nonce, amount) = multi_arg.into_tuple();

            all_payments.push(DctTokenPayment {
                token_identifier: token_id,
                token_nonce: nonce,
                amount,
                token_type: DctTokenType::Invalid,
            });
        }

        self.send()
            .transfer_multiple_dct_via_async_call(&caller, &all_payments, b"");
    }

    #[payable("*")]
    #[endpoint]
    fn burn_and_create_retrive_async(&self) {
        let payments = self.call_value().all_dct_transfers();
        let mut uris = ManagedVec::new();
        uris.push(ManagedBuffer::new());

        let mut new_tokens = ManagedVec::new();

        for payment in payments.into_iter() {
            // burn old tokens
            self.send().dct_local_burn(
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            );

            // create new ones
            let new_token_nonce = self.send().dct_nft_create(
                &payment.token_identifier,
                &payment.amount,
                &ManagedBuffer::new(),
                &BigUint::zero(),
                &ManagedBuffer::new(),
                &Empty,
                &uris,
            );

            new_tokens.push(DctTokenPayment {
                token_identifier: payment.token_identifier,
                token_nonce: new_token_nonce,
                amount: payment.amount,
                token_type: DctTokenType::Invalid, // ignored
            });
        }

        self.send().transfer_multiple_dct_via_async_call(
            &self.blockchain().get_caller(),
            &new_tokens,
            &[],
        );
    }

    /// TODO: invert token_payment and token_nonce, for consistency.
    #[event("accept_funds")]
    fn accept_funds_event(
        &self,
        #[indexed] moax_value: &BigUint,
        #[indexed] multi_dct: &MultiValueEncoded<DctTokenPaymentMultiValue>,
    );

    #[event("reject_funds")]
    fn reject_funds_event(
        &self,
        #[indexed] moax_value: &BigUint,
        #[indexed] multi_dct: &MultiValueEncoded<DctTokenPaymentMultiValue>,
    );

    #[event("retrieve_funds")]
    fn retrieve_funds_event(
        &self,
        #[indexed] token: &TokenIdentifier,
        #[indexed] nonce: u64,
        #[indexed] amount: &BigUint,
    );

    #[endpoint]
    fn get_owner_address(&self) -> ManagedAddress {
        self.blockchain().get_owner_address()
    }

    /// We already leave a trace of the calls using the event logs;
    /// this additional counter has the role of showing that storage also gets saved correctly.
    #[view]
    #[storage_mapper("call_counts")]
    fn call_counts(&self, endpoint: ManagedBuffer) -> SingleValueMapper<usize>;
}
