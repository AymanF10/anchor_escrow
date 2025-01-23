use anchor_lang::prelude::*;

/// Defines the data stored for an escrow, which includes:
/// - a seed,
/// - maker's public key,
/// - token types (`mint_a` and `mint_b`),
/// - the expected receive amount,
/// - and a bump seed for address generation security.
#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seed: u64,      // seed for the escrow account
    pub maker: Pubkey,  // maker of the trade
    pub mint_a: Pubkey, // token that the maker is expected to deposit
    pub mint_b: Pubkey, // token that the maker is expecting to receive
    pub receive: u64,   // amount of mint_b that the maker is expecting to receive
    pub bump: u8,       // bump seed for the escrow account
}