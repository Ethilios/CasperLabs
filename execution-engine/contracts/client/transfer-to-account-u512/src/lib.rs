#![no_std]

use contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{account::AccountHash, ApiError, U512};

#[repr(u16)]
enum Args {
    AccountHash = 0,
    Amount = 1,
}

#[repr(u32)]
enum CustomError {
    MissingAccountAccountHash = 1,
    InvalidAccountAccountHash = 2,
    MissingAmount = 3,
    InvalidAmount = 4,
}

/// Executes mote transfer to supplied public key.
/// Transfers the requested amount.
#[no_mangle]
pub fn delegate() {
    let account_hash: AccountHash = runtime::get_arg(Args::AccountHash as u32)
        .unwrap_or_revert_with(ApiError::User(CustomError::MissingAccountAccountHash as u16))
        .unwrap_or_revert_with(ApiError::User(CustomError::InvalidAccountAccountHash as u16));
    let transfer_amount: U512 = runtime::get_arg(Args::Amount as u32)
        .unwrap_or_revert_with(ApiError::User(CustomError::MissingAmount as u16))
        .unwrap_or_revert_with(ApiError::User(CustomError::InvalidAmount as u16));
    system::transfer_to_account(account_hash, transfer_amount).unwrap_or_revert();
}
