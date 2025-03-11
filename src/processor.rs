use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{instruction::transfer_checked, state::Mint};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // Accounts need to transfer
    let source_info = next_account_info(account_info_iter)?; // 1.
    let mint_info = next_account_info(account_info_iter)?; // 2.
    let destination_info = next_account_info(account_info_iter)?; // 3.
    let authority_info = next_account_info(account_info_iter)?; // 4.
    let token_program_info = next_account_info(account_info_iter)?; // 5.

    // check PDA Account by finding 'authority'
    let (expected_authority, bump_seed) = Pubkey::find_program_address(&[b"authority"], program_id);
    if expected_authority != *authority_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // get transfer value from trancfer instruction
    let transfer_amount = u64::from_le_bytes(instruction_data.try_into().unwrap());

    // The program uses `transfer_checked`, which requires the number of decimals from Mint Account
    let mint = Mint::unpack(&mint_info.try_borrow_data()?)?;
    let token_decimals = mint.decimals;

    // Invoke Token Transfer
    msg!("Attempting to transfer {} tokens", transfer_amount);
    invoke_signed(
        &transfer_checked(
            token_program_info.key,
            source_info.key,
            mint_info.key,
            destination_info.key,
            authority_info.key,
            &[], // no multisig allowed
            transfer_amount,
            token_decimals,
        )
        .unwrap(),
        &[
            source_info.clone(),
            mint_info.clone(),
            destination_info.clone(),
            authority_info.clone(),
            token_program_info.clone(), // not required, but better for clarity
        ],
        // use bump_seed as PDA sign
        &[&[b"authority", &[bump_seed]]],
    )
}
