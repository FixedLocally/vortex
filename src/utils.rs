use solana_program::pubkey::Pubkey;

pub fn pubkey_from_slice(slice: &[u8]) -> Pubkey {
    Pubkey::new_from_array(slice.try_into().expect("slice with incorrect length"))
}