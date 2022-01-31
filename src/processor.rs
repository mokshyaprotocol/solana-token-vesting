//! Program state processor

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    sysvar::{Sysvar,rent::Rent,clock::Clock},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    system_program,
};

use crate::{
    instruction::{ProcessDepositToken,ProcessUnlock, ProcessWinner, TokenInstruction},
    state::PDA,
    utils::{assert_keys_equal, get_master_address_and_bump_seed,create_pda_account},
};


use spl_associated_token_account::get_associated_token_address;

pub struct Processor {}

impl Processor {
    pub fn process_deposit_token(program_id: &Pubkey,accounts: &[AccountInfo],amount: u64,end_time:u64) -> ProgramResult {


        let account_info_iter = &mut accounts.iter();
        let sender_account = next_account_info(account_info_iter)?; // sender
        let pda = next_account_info(account_info_iter)?; // pda account
        let token_program = next_account_info(account_info_iter)?;
        let token_mint = next_account_info(account_info_iter)?; ////////// token mint address
        let system_program = next_account_info(account_info_iter)?;
        let rent_account = next_account_info(account_info_iter)?;
        let pda_associated_info = next_account_info(account_info_iter)?; // Associated token of pda
        let associated_token_info = next_account_info(account_info_iter)?; // Associated token master
        let associated_token_address = next_account_info(account_info_iter)?;
        let pda_data =next_account_info(account_info_iter)?; //account to save data 
        let receiver=next_account_info(account_info_iter)?; //account to save data 

        //Was the transaction signed by sender account's private key
        if !sender_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let (account_address, _bump_seed) = get_master_address_and_bump_seed(
            receiver.key,
            program_id,
        );
        let pda_token = get_associated_token_address(&account_address, token_mint.key); //creating pda token account as per token available

        //comparing admin_pda and pda
        //comparing mint addresses
        //return error
        if  spl_token::id() != *token_program.key
            && pda_token != *pda_associated_info.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let now:u64 = Clock::get()?.unix_timestamp as u64;
        if now>=end_time
        {
            return Err(ProgramError::MissingRequiredSignature);
        } 
        if pda_associated_info.data_is_empty() {
            invoke(
                &spl_associated_token_account::create_associated_token_account(
                    sender_account.key,
                    pda.key,
                    token_mint.key,
                ),
                &[
                    sender_account.clone(),
                    pda_associated_info.clone(),
                    pda.clone(),
                    token_mint.clone(),
                    token_program.clone(),
                    rent_account.clone(),
                    associated_token_info.clone(),
                    system_program.clone(),
                ],
            )?
        }

        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                associated_token_address.key,
                pda_associated_info.key,
                sender_account.key,
                &[sender_account.key],
                amount,
            )?,
            &[
                token_program.clone(),
                associated_token_address.clone(),
                pda_associated_info.clone(),
                sender_account.clone(),
                system_program.clone(),
            ],
        )?;
        let rent = Rent::get()?;
        let transfer_amount =  rent.minimum_balance (std::mem::size_of::<PDA>());
        // Sending transaction fee to recipient. So, he can withdraw the streamed fund
        
        create_pda_account( 
            sender_account,
            transfer_amount,
            std::mem::size_of::<PDA>(),
            program_id,
            system_program,
            pda_data
        )?;

        let mut pda_deposit = PDA::try_from_slice(&pda_data.data.borrow())?;
        
        pda_deposit.amount = amount; //amount deposited
        pda_deposit.start_time=now;
        pda_deposit.end_time=end_time;
        pda_deposit.sender_account = *sender_account.key;
        pda_deposit.pda = *pda.key;
        pda_deposit.mint_address = *token_mint.key;
        
        pda_deposit.serialize(&mut &mut pda_data.data.borrow_mut()[..])?;
        
        Ok(())
    }
    
    pub fn unlock_token(program_id: &Pubkey,accounts: &[AccountInfo],random:u64)->ProgramResult
    {  
        let account_info_iter = &mut accounts.iter();
        let receiver=next_account_info(account_info_iter)?;
        let sender_account = next_account_info(account_info_iter)?; // sender
        let pda = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let pda_data = next_account_info(account_info_iter)?; // pda
        let token_program = next_account_info(account_info_iter)?;
        let token_mint = next_account_info(account_info_iter)?; ////////// token mint address
        let rent_account = next_account_info(account_info_iter)?;
        let pda_associated_info = next_account_info(account_info_iter)?; // Associated token address of pda
        let associated_token_info = next_account_info(account_info_iter)?; // Associated token master of the giver
        let associated_token_address = next_account_info(account_info_iter)?; //receiver associated token address
        let receiver_token = get_associated_token_address(&receiver.key, token_mint.key); //receiver token account as per token available


        let mut pda_deposit = PDA::try_from_slice(&pda_data.data.borrow())?;

        //checking if sender and admin are valid or not
        if *sender_account.key !=pda_deposit.sender_account && *pda.key!=pda_deposit.pda
         {
            return Err(ProgramError::MissingRequiredSignature);
         
        }
        let (account_address, bump_seed) = get_master_address_and_bump_seed(
            receiver_account.key,
            program_id,
        );
        let pda_signer_seeds: &[&[_]] = &[
            &sender_account.key.to_bytes(),
            &[bump_seed],
        ];
        if  spl_token::id() != *token_program.key
            && receiver_token != *associated_token_address.key
            && *associated_token_info.key != *pda_associated_info.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let now:u64 = Clock::get()?.unix_timestamp as u64;
        if pda_deposit.end_time <=now && pda_deposit.amount>0
        {
        if associated_token_address.data_is_empty() {
            invoke_signed(
                &spl_associated_token_account::create_associated_token_account(
                    pda.key,
                    receiver.key,
                    token_mint.key,
                ),
                &[
                    pda_associated_info.clone(),
                    pda.clone(),
                    token_mint.clone(),
                    token_program.clone(),
                    rent_account.clone(),
                    associated_token_info.clone(),
                    associated_token_address.clone(),
                    system_program.clone(),
                ],&[&pda_signer_seeds],
            )?
        }
            let send_amount = pda_deposit.amount;
            pda_deposit.amount=0;

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                pda_associated_info.key,
                associated_token_address.key,
                pda.key,
                &[pda.key],
                send_amount,
            )?,
            &[
                token_program.clone(),
                pda_associated_info.clone(),
                associated_token_address.clone(),
                pda.clone(),
                system_program.clone()
            ],&[&pda_signer_seeds],
        )?;
    }

    Ok(())
       

    }

        
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;
        match instruction {
            TokenInstruction::ProcessDepositToken(ProcessDepositToken { amount}) => {
                msg!("Instruction: Deposit token");
                Self::process_deposit_token(program_id, accounts, amount)
            }
            TokenInstruction::ProcessWinner(ProcessWinner{ random }) => {
                msg!("Instruction: Check For winner");
                Self::unlock_token(program_id, accounts, random)
            }
        }
    }
}
