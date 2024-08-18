use solana_sdk::pubkey::Pubkey;

pub struct PubkeyPair {
    pub pubkey_a: Pubkey,
    pub pubkey_b: Pubkey
}

pub enum Pool {
    ORCA,
}

pub trait PoolOperation {
    fn unpack_data(data: &Vec<u8>) -> Self;
}