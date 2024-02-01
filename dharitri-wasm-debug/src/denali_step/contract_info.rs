use std::ops::{Deref, DerefMut};

use dharitri_wasm::{
    api::ManagedTypeApi,
    contract_base::ProxyObjBase,
    dharitri_codec::{CodecFrom, EncodeErrorHandler, TopEncode, TopEncodeOutput},
    types::{Address, ManagedAddress},
};
use denali::{
    interpret_trait::{InterpretableFrom, InterpreterContext},
    model::{AddressKey, AddressValue},
};

/// Bundles a denali representation of a contract with the contract proxy,
/// so that it can be easily called in the context of a blockchain mock.
pub struct ContractInfo<P: ProxyObjBase> {
    pub denali_address_expr: AddressKey,
    proxy_inst: P,
}

impl<P: ProxyObjBase> ContractInfo<P> {
    pub fn new<A>(address_expr: A, ic: &InterpreterContext) -> Self
    where
        AddressKey: InterpretableFrom<A>,
    {
        let denali_address_expr = AddressKey::interpret_from(address_expr, ic);
        let proxy_inst = P::new_proxy_obj().contract(denali_address_expr.value.clone().into());
        ContractInfo {
            denali_address_expr,
            proxy_inst,
        }
    }

    pub fn to_address(&self) -> Address {
        self.denali_address_expr.to_address()
    }
}

impl<P: ProxyObjBase> InterpretableFrom<&ContractInfo<P>> for AddressKey {
    fn interpret_from(from: &ContractInfo<P>, _context: &InterpreterContext) -> Self {
        from.denali_address_expr.clone()
    }
}

impl<P: ProxyObjBase> InterpretableFrom<&ContractInfo<P>> for AddressValue {
    fn interpret_from(from: &ContractInfo<P>, context: &InterpreterContext) -> Self {
        AddressValue::interpret_from(&from.denali_address_expr, context)
    }
}

impl<P: ProxyObjBase> Deref for ContractInfo<P> {
    type Target = P;
    fn deref(&self) -> &Self::Target {
        &self.proxy_inst
    }
}

impl<P: ProxyObjBase> DerefMut for ContractInfo<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let proxy_inst = core::mem::replace(&mut self.proxy_inst, P::new_proxy_obj());
        let proxy_inst = proxy_inst.contract(self.denali_address_expr.value.clone().into());
        let _ = core::mem::replace(&mut self.proxy_inst, proxy_inst);
        &mut self.proxy_inst
    }
}

impl<P: ProxyObjBase> TopEncode for ContractInfo<P> {
    fn top_encode_or_handle_err<O, H>(&self, output: O, h: H) -> Result<(), H::HandledErr>
    where
        O: TopEncodeOutput,
        H: EncodeErrorHandler,
    {
        self.denali_address_expr
            .value
            .top_encode_or_handle_err(output, h)
    }
}

impl<P: ProxyObjBase> CodecFrom<ContractInfo<P>> for Address {}
impl<P: ProxyObjBase> CodecFrom<&ContractInfo<P>> for Address {}
impl<M: ManagedTypeApi, P: ProxyObjBase> CodecFrom<ContractInfo<P>> for ManagedAddress<M> {}
impl<M: ManagedTypeApi, P: ProxyObjBase> CodecFrom<&ContractInfo<P>> for ManagedAddress<M> {}
