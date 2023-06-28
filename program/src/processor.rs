//! Program processor

use {
    crate::{instruction::KeyringProgramInstruction, state::Keystore},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
    },
};

fn check_authority(authority_info: &AccountInfo) -> ProgramResult {
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// Processes a `CreateKeystore` instruction.
pub fn process_create_keystore(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let keystore_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;

    let bump_seed = {
        check_authority(authority_info)?;
        Keystore::check_pda(program_id, authority_info.key, keystore_info.key)?
    };

    let mut signer_seeds = Keystore::seeds(authority_info.key);
    let bump_signer_seed = [bump_seed];
    signer_seeds.push(&bump_signer_seed);

    invoke_signed(
        &system_instruction::create_account(
            authority_info.key,
            keystore_info.key,
            Rent::default().minimum_balance(0),
            0u64,
            program_id,
        ),
        &[authority_info.clone(), keystore_info.clone()],
        &[&signer_seeds],
    )?;

    Ok(())
}

/// Processes a `AddKey` instruction.
pub fn process_add_entry(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    add_entry_data: Vec<u8>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let keystore_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;

    {
        Keystore::check_pda(program_id, authority_info.key, keystore_info.key)?;
        check_authority(authority_info)?;
    }

    Keystore::add_entry(keystore_info, &add_entry_data)?;

    Ok(())
}

/// Processes a `RemoveKey` instruction.
pub fn process_remove_entry(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    remove_entry_data: Vec<u8>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let keystore_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;

    {
        Keystore::check_pda(program_id, authority_info.key, keystore_info.key)?;
        check_authority(authority_info)?;
    }

    Keystore::remove_entry(keystore_info, &remove_entry_data)?;

    Ok(())
}

/// Processes a `KeyringProgramInstruction` instruction.
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = KeyringProgramInstruction::unpack(input)?;

    match instruction {
        KeyringProgramInstruction::CreateKeystore {} => {
            msg!("Instruction: CreateKeystore");
            process_create_keystore(program_id, accounts)
        }
        KeyringProgramInstruction::AddEntry { add_entry_data } => {
            msg!("Instruction: AddKey");
            process_add_entry(program_id, accounts, add_entry_data)
        }
        KeyringProgramInstruction::RemoveEntry { remove_entry_data } => {
            msg!("Instruction: RemoveKey");
            process_remove_entry(program_id, accounts, remove_entry_data)
        }
    }
}
