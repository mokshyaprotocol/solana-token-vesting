
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke,
    pubkey::Pubkey,
    system_instruction,
};

pub fn address_and_bump_seed(sender: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&sender.to_bytes()], program_id)
}

pub fn create_pda<'a>(
    payer: &AccountInfo<'a>,
    amount: u64,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
) -> ProgramResult {
    invoke(
        &system_instruction::create_account(
            payer.key,
            new_pda_account.key,
            amount,
            space as u64,
            owner,
        ),
        &[
            payer.clone(),
            new_pda_account.clone(),
            system_program.clone(),
        ],
    )
}

