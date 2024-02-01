#![feature(generic_associated_types)]
#![allow(clippy::type_complexity)]
#![feature(exhaustive_patterns)]

pub mod abi_json;
pub mod api;
mod contract_map;
mod display_util;
mod managed_test_util;
mod denali_go_runner;
mod denali_rs_runner;
mod denali_step;
pub mod meta;
pub mod testing_framework;
pub mod tx_execution;
pub mod tx_mock;
pub mod world_mock;

pub use contract_map::*;
pub use display_util::*;
pub use managed_test_util::*;
pub use denali_step::*;

pub use denali_go_runner::denali_go;
pub use denali_rs_runner::denali_rs;
pub use tx_mock::DebugApi;
pub use world_mock::BlockchainMock;

// Re-exporting the whole denali crate for easier use in tests.
pub use denali;

// Re-exporting for convenience. Using the crate as imported in the codec to make sure the save version is used everywhere.
pub use dharitri_wasm::dharitri_codec::num_bigint;

#[macro_use]
extern crate alloc;
pub use alloc::{boxed::Box, vec::Vec};

pub use std::collections::HashMap;
