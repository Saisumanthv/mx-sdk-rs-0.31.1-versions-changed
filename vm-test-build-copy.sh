#!/bin/bash

# copies wasm & denali files to the Arwen test folder
# expects 1 argument: the path to the Arwen repo root

VM_REPO_PATH=${1:?"Missing VM repo path!"}

build_and_copy() {
   contract_path=$1
   contract_name=${contract_path##*/}
   vm_contract_path=$2

   erdpy --verbose contract build --skip-eei-checks $contract_path || return 1
   mkdir -p $vm_contract_path/output
   cp $contract_path/output/$contract_name.wasm \
      $vm_contract_path/output/$contract_name.wasm
   cp $contract_path/output/$contract_name-view.wasm \
      $vm_contract_path/output/$contract_name-view.wasm
   cp -R $contract_path/denali \
      $vm_contract_path
}

# building all contracts takes a lot of time, only the ones for the wasm-vm tests are built below
# if you still want to build all:
# ./build-wasm.sh

build_and_copy ./contracts/examples/adder $VM_REPO_PATH/test/adder
build_and_copy ./contracts/examples/crowdfunding-dct $VM_REPO_PATH/test/crowdfunding-dct
build_and_copy ./contracts/examples/digital-cash $VM_REPO_PATH/test/digital-cash
build_and_copy ./contracts/examples/ping-pong-moax $VM_REPO_PATH/test/ping-pong-moax
build_and_copy ./contracts/experimental/multisig-external-view $VM_REPO_PATH/test/multisig
build_and_copy ./contracts/examples/moax-dct-swap $VM_REPO_PATH/test/moax-dct-swap
build_and_copy ./contracts/feature-tests/alloc-features $VM_REPO_PATH/test/features/alloc-features
build_and_copy ./contracts/feature-tests/basic-features $VM_REPO_PATH/test/features/basic-features
build_and_copy ./contracts/feature-tests/erc-style-contracts/erc20 $VM_REPO_PATH/test/erc20-rust
build_and_copy ./contracts/feature-tests/payable-features $VM_REPO_PATH/test/features/payable-features
build_and_copy ./contracts/feature-tests/dct-system-sc-mock $VM_REPO_PATH/test/features/dct-system-sc-mock

build_and_copy_composability() {
   contract=$1
   contract_with_underscores="${contract//-/_}"

   # with managed-ei
   erdpy --verbose contract build --skip-eei-checks ./contracts/feature-tests/composability/$contract || return 1
   cp -R contracts/feature-tests/composability/$contract/output/${contract}.wasm \
      $VM_REPO_PATH/test/features/composability/$contract/output/${contract}.wasm

   # without managed-ei
   export RUSTFLAGS=${RUSTFLAGS-'-C link-arg=-s'}
   cd contracts/feature-tests/composability/$contract/wasm-no-managed-ei
   cargo build --target=wasm32-unknown-unknown --release
   cd ..
   mkdir -p output
   cp \
      wasm-no-managed-ei/target/wasm32-unknown-unknown/release/${contract_with_underscores}_wasm.wasm \
      output/${contract}-unmanaged.wasm
   cd ../../../..

   cp -R contracts/feature-tests/composability/$contract/output/${contract}-unmanaged.wasm \
      $VM_REPO_PATH/test/features/composability/$contract/output/${contract}-unmanaged.wasm
}

build_and_copy_composability forwarder
build_and_copy_composability forwarder-raw
build_and_copy_composability proxy-test-first
build_and_copy_composability proxy-test-second
build_and_copy_composability recursive-caller

erdpy --verbose contract build --skip-eei-checks ./contracts/feature-tests/composability/vault || return 1
cp -R contracts/feature-tests/composability/vault/output/vault.wasm \
   $VM_REPO_PATH/test/features/composability/vault/output/vault.wasm
# erdpy --verbose  contract build --skip-eei-checks ./contracts/feature-tests/composability/promises-features || return 1
# cp -R contracts/feature-tests/composability/promises-features/output/promises-features.wasm \
#    $VM_REPO_PATH/test/features/composability/promises-features/output/promises-features.wasm

cp -R contracts/feature-tests/composability/denali \
   $VM_REPO_PATH/test/features/composability
rm -f $VM_REPO_PATH/test/features/composability/denali-legacy/*
mmv -c 'contracts/feature-tests/composability/denali/*.scen.json' \
   $VM_REPO_PATH/test/features/composability/denali-legacy/l_'#1.scen.json'

sed -i 's/forwarder.wasm/forwarder-unmanaged.wasm/g' $VM_REPO_PATH/test/features/composability/denali-legacy/*
sed -i 's/forwarder-raw.wasm/forwarder-raw-unmanaged.wasm/g' $VM_REPO_PATH/test/features/composability/denali-legacy/*
sed -i 's/proxy-test-first.wasm/proxy-test-first-unmanaged.wasm/g' $VM_REPO_PATH/test/features/composability/denali-legacy/*
sed -i 's/proxy-test-second.wasm/proxy-test-second-unmanaged.wasm/g' $VM_REPO_PATH/test/features/composability/denali-legacy/*
sed -i 's/recursive-caller.wasm/recursive-caller-unmanaged.wasm/g' $VM_REPO_PATH/test/features/composability/denali-legacy/*
sed -i 's/proxy_test_init.scen.json/l_proxy_test_init.scen.json/g' $VM_REPO_PATH/test/features/composability/denali-legacy/*
