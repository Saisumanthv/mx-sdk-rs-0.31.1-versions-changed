use core::marker::PhantomData;

use crate::{
    api::{
        const_handles, CallValueApi, CallValueApiImpl, ErrorApi, ErrorApiImpl, ManagedBufferApi,
        ManagedTypeApi,
    },
    err_msg,
    types::{BigUint, DctTokenPayment, DctTokenType, ManagedType, ManagedVec, TokenIdentifier},
};

#[derive(Default)]
pub struct CallValueWrapper<A>
where
    A: CallValueApi + ErrorApi + ManagedTypeApi,
{
    _phantom: PhantomData<A>,
}

impl<A> CallValueWrapper<A>
where
    A: CallValueApi + ErrorApi + ManagedTypeApi,
{
    pub fn new() -> Self {
        CallValueWrapper {
            _phantom: PhantomData,
        }
    }

    /// Retrieves the MOAX call value from the VM.
    /// Will return 0 in case of an DCT transfer (cannot have both MOAX and DCT transfer simultaneously).
    pub fn moax_value(&self) -> BigUint<A> {
        A::call_value_api_impl().load_moax_value(const_handles::CALL_VALUE_MOAX);
        BigUint::from_raw_handle(const_handles::CALL_VALUE_MOAX) // unsafe, TODO: replace with ManagedRef<...>
    }

    /// Returns all DCT transfers that accompany this SC call.
    /// Will return 0 results if nothing was transfered, or just MOAX.
    /// Fully managed underlying types, very efficient.
    pub fn all_dct_transfers(&self) -> ManagedVec<A, DctTokenPayment<A>> {
        A::call_value_api_impl().load_all_dct_transfers(const_handles::CALL_VALUE_MULTI_DCT);
        ManagedVec::from_raw_handle(const_handles::CALL_VALUE_MULTI_DCT) // unsafe, TODO: replace with ManagedRef<...>
    }

    /// Retrieves the DCT call value from the VM.
    /// Will return 0 in case of an MOAX transfer (cannot have both MOAX and DCT transfer simultaneously).
    /// Warning, not tested with multi transfer, use `all_dct_transfers` instead!
    pub fn dct_value(&self) -> BigUint<A> {
        A::call_value_api_impl().load_single_dct_value(const_handles::CALL_VALUE_SINGLE_DCT);
        BigUint::from_raw_handle(const_handles::CALL_VALUE_SINGLE_DCT)
    }

    /// Returns the call value token identifier of the current call.
    /// The identifier is wrapped in a TokenIdentifier object, to hide underlying logic.
    ///
    /// A note on implementation: even though the underlying api returns an empty name for MOAX,
    /// but the MOAX TokenIdentifier is serialized as `MOAX`.
    /// Warning, not tested with multi transfer, use `all_dct_transfers` instead!
    pub fn token(&self) -> TokenIdentifier<A> {
        let call_value_api = A::call_value_api_impl();
        if call_value_api.dct_num_transfers() == 0 {
            TokenIdentifier::moax()
        } else {
            TokenIdentifier::from_raw_handle(call_value_api.token())
        }
    }

    /// Returns the nonce of the received DCT token.
    /// Will return 0 in case of MOAX or fungible DCT transfer.
    /// Warning, not tested with multi transfer, use `all_dct_transfers` instead!
    pub fn dct_token_nonce(&self) -> u64 {
        let call_value_api = A::call_value_api_impl();
        if call_value_api.dct_num_transfers() > 0 {
            call_value_api.dct_token_nonce()
        } else {
            0
        }
    }

    /// Returns the DCT token type.
    /// Will return "Fungible" for MOAX.
    /// Warning, not tested with multi transfer, use `all_dct_transfers` instead!
    pub fn dct_token_type(&self) -> DctTokenType {
        let call_value_api = A::call_value_api_impl();
        if call_value_api.dct_num_transfers() > 0 {
            A::call_value_api_impl().dct_token_type()
        } else {
            DctTokenType::Fungible
        }
    }

    pub fn require_moax(&self) -> BigUint<A> {
        let call_value_api = A::call_value_api_impl();
        if call_value_api.dct_num_transfers() > 0 {
            A::error_api_impl().signal_error(err_msg::NON_PAYABLE_FUNC_DCT.as_bytes());
        }

        self.moax_value()
    }

    pub fn require_dct(&self, token: &[u8]) -> BigUint<A> {
        let m_api = A::managed_type_impl();
        let call_value_api = A::call_value_api_impl();
        let error_api = A::error_api_impl();

        let expected_token_handle = const_handles::MBUF_TEMPORARY_1;
        m_api.mb_overwrite(expected_token_handle, token);
        if call_value_api.dct_num_transfers() != 1 {
            error_api.signal_error(err_msg::SINGLE_DCT_EXPECTED.as_bytes());
        }
        if !m_api.mb_eq(call_value_api.token(), expected_token_handle) {
            error_api.signal_error(err_msg::BAD_TOKEN_PROVIDED.as_bytes());
        }
        call_value_api.load_single_dct_value(const_handles::CALL_VALUE_SINGLE_DCT);
        BigUint::from_raw_handle(const_handles::CALL_VALUE_SINGLE_DCT)
    }

    /// Returns both the call value (either MOAX or DCT) and the token identifier.
    /// Especially used in the `#[payable("*")] auto-generated snippets.
    /// TODO: replace with multi transfer handling everywhere
    pub fn payment_token_pair(&self) -> (BigUint<A>, TokenIdentifier<A>) {
        let call_value_api = A::call_value_api_impl();
        if call_value_api.dct_num_transfers() == 0 {
            (self.moax_value(), TokenIdentifier::moax())
        } else {
            (self.dct_value(), self.token())
        }
    }

    pub fn payment(&self) -> DctTokenPayment<A> {
        let api = A::call_value_api_impl();
        if api.dct_num_transfers() == 0 {
            DctTokenPayment::new(TokenIdentifier::moax(), 0, self.moax_value())
        } else {
            DctTokenPayment::new(self.token(), self.dct_token_nonce(), self.dct_value())
        }
    }

    pub fn payment_as_tuple(&self) -> (TokenIdentifier<A>, u64, BigUint<A>) {
        let (amount, token) = self.payment_token_pair();
        let nonce = self.dct_token_nonce();

        (token, nonce, amount)
    }
}
