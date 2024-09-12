use arrayref::{array_ref, array_refs};
use num_enum::{TryFromPrimitive};
use solana_sdk::program_error::ProgramError;
use solana_sdk::program_option::COption;
use solana_sdk::pubkey::Pubkey;
use crate::account::account::AccountDataSerializer;

#[repr(u8)]
#[derive(Eq, PartialEq, TryFromPrimitive)]
pub enum AccountState {
    Uninitialized = 0,
    Initialized = 1,
    Frozen = 2,
}

pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub delegate: COption<Pubkey>,
    pub state: AccountState,
    pub is_native: COption<u64>,
    pub delegated_amount: u64,
    pub close_authority: COption<Pubkey>
}

impl AccountDataSerializer for TokenAccount {
    fn unpack_data(data: &Vec<u8>) -> TokenAccount {
        let src = array_ref![data, 0, 165];
        let (mint, owner, amount, delegate, state, is_native, delegated_amount, close_authority) =
            array_refs![src, 32, 32, 8, 36, 1, 12, 8, 36];

        TokenAccount {
            mint: Pubkey::new_from_array(*mint),
            owner: Pubkey::new_from_array(*owner),
            amount: u64::from_le_bytes(*amount),
            delegate: Self::unpack_coption_key(delegate).unwrap(),
            state: AccountState::try_from_primitive(u8::from_le_bytes(*state)).or(Err(ProgramError::InvalidAccountData)).unwrap(),
            is_native: Self::unpack_coption_u64(is_native).unwrap(),
            delegated_amount: u64::from_le_bytes(*delegated_amount),
            close_authority: Self::unpack_coption_key(close_authority).unwrap(),
        }
    }
}

impl TokenAccount {
    fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>, ProgramError> {
        let (tag, body) = array_refs![src, 4, 32];
        match *tag {
            [0, 0, 0, 0] => Ok(COption::None),
            [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    fn unpack_coption_u64(src: &[u8; 12]) -> Result<COption<u64>, ProgramError> {
        let (tag, body) = array_refs![src, 4, 8];
        match *tag {
            [0, 0, 0, 0] => Ok(COption::None),
            [1, 0, 0, 0] => Ok(COption::Some(u64::from_le_bytes(*body))),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}