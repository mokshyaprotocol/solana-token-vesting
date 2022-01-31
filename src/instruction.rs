//! Instruction types
use crate::error::TokenError;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

pub struct ProcessDepositToken {
    pub amount: u64,
    pub end_time:u64,
}

pub struct ProcessUnlock {
    pub amount: u64

}

pub enum TokenInstruction {
    ProcessDepositToken(ProcessDepositToken),
    ProcessUnlock(ProcessUnlock),
}

impl TokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (amount, rest) = rest.split_at(8);
                let amount = amount
                    .try_into()
                    .map(u64::from_le_bytes)
                .or(Err(InvalidInstruction))?;  
                let (end_time, _rest) = rest.split_at(8);
                let end_time = end_time
                    .try_into()
                    .map(u64::from_le_bytes)
                    .or(Err(InvalidInstruction))?;                   
                Self::ProcessDepositToken(ProcessDepositToken{amount,end_time})
            }
            1 => {
                let (amount, _rest) = rest.split_at(8);
                let amount = amount
                    .try_into()
                    .map(u64::from_le_bytes)
                    .or(Err(InvalidInstruction))?;                
                Self::ProcessUnlock(ProcessUnlock{amount})
            }
            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }
}
