#![no_std]

extern crate contract_ffi;

use contract_ffi::contract_api::{account, runtime, Error};
use contract_ffi::unwrap_or_revert::UnwrapOrRevert;
use contract_ffi::value::account::{ActionType, PublicKey, Weight};

#[no_mangle]
pub extern "C" fn call() {
    account::add_associated_key(PublicKey::new([123; 32]), Weight::new(254))
        .unwrap_or_else(|_| runtime::revert(Error::User(50)));
    let key_management_threshold: Weight = runtime::get_arg(0)
        .unwrap_or_revert_with(Error::MissingArgument)
        .unwrap_or_revert_with(Error::InvalidArgument);

    let deployment_threshold: Weight = runtime::get_arg(1)
        .unwrap_or_revert_with(Error::MissingArgument)
        .unwrap_or_revert_with(Error::InvalidArgument);

    account::set_action_threshold(ActionType::KeyManagement, key_management_threshold)
        .unwrap_or_else(|_| runtime::revert(Error::User(100)));
    account::set_action_threshold(ActionType::Deployment, deployment_threshold)
        .unwrap_or_else(|_| runtime::revert(Error::User(200)));
}
