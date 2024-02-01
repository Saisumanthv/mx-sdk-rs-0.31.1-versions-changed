#![feature(generic_associated_types)]

use dharitri_wasm::{
    api::ManagedTypeApi,
    derive::ManagedVecItem,
    dharitri_codec,
    dharitri_codec::dharitri_codec_derive::{NestedDecode, NestedEncode, TopDecode, TopEncode},
    types::{BigUint, ManagedType},
};
use dharitri_wasm_debug::DebugApi;

// to test, run the following command in dharitri-wasm-debug folder:
// cargo expand --test derive_managed_vec_item_biguint_test > expanded.rs

#[derive(
    ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Clone, Debug,
)]
pub struct ManagedStructWithBigUint<M: ManagedTypeApi> {
    pub big_uint: dharitri_wasm::types::BigUint<M>,
    pub num: u32,
}

#[test]
fn struct_with_numbers_static() {
    assert_eq!(
        <ManagedStructWithBigUint<DebugApi> as dharitri_wasm::types::ManagedVecItem>::PAYLOAD_SIZE,
        8
    );
    assert!(
        !<ManagedStructWithBigUint<DebugApi> as dharitri_wasm::types::ManagedVecItem>::SKIPS_RESERIALIZATION
    );
}

#[test]
fn managed_struct_to_bytes_writer() {
    let _ = DebugApi::dummy();
    let fortytwo = 42u64;
    let s = ManagedStructWithBigUint::<DebugApi> {
        big_uint: BigUint::from(fortytwo),
        num: 0x12345,
    };
    let mut arr: [u8; 8] = [0u8;
        <ManagedStructWithBigUint<DebugApi> as dharitri_wasm::types::ManagedVecItem>::PAYLOAD_SIZE];

    let handle_bytes = s.big_uint.get_raw_handle().to_be_bytes();
    let expected = [0xff, 0xff, 0xff, handle_bytes[3], 0x00, 0x01, 0x23, 0x45];

    <ManagedStructWithBigUint<DebugApi> as dharitri_wasm::types::ManagedVecItem>::to_byte_writer(
        &s,
        |bytes| {
            arr[0..<ManagedStructWithBigUint::<DebugApi> as dharitri_wasm::types::ManagedVecItem>::PAYLOAD_SIZE].copy_from_slice(bytes);

            assert_eq!(arr, expected);
        },
    );
}

#[test]
fn managed_struct_from_bytes_reader() {
    let _ = DebugApi::dummy();
    let s = ManagedStructWithBigUint::<DebugApi> {
        big_uint: BigUint::from(42u64),
        num: 0x12345,
    };
    let handle_bytes = s.big_uint.get_raw_handle().to_be_bytes();
    let arr: [u8; 8] = [0xff, 0xff, 0xff, handle_bytes[3], 0x00, 0x01, 0x23, 0x45];

    let struct_from_bytes = <ManagedStructWithBigUint<DebugApi> as dharitri_wasm::types::ManagedVecItem>::from_byte_reader( |bytes| {
        bytes.copy_from_slice(
                    &arr
                        [0
                            ..<ManagedStructWithBigUint::<DebugApi> as dharitri_wasm::types::ManagedVecItem>::PAYLOAD_SIZE],
                );
    });
    assert_eq!(s, struct_from_bytes);
}
