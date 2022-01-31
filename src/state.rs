///into state.rs
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
//deposit tokens
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct PDA {
    pub amount: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub pda: Pubkey,        
    pub sender_account: Pubkey,
    pub mint_address: Pubkey,
    pub receiver: Pubkey,
}
