use anchor_lang::prelude::*;

declare_id!("73bhLRuwcNwd7prQnWqGrSARy5ozmKykGEUBPzwCSGgS");

pub mod state;
pub use state::*;
pub mod instructions;
pub use instructions::*;

#[program]
pub mod escrow {
    use super::*;

    /// Initiates the process of making an escrow
    /// Takes a seed, deposit amount, and receive amount
    /// Designed to deposit funds and set up the escrow conditions
    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.deposit(deposit)?;
        ctx.accounts.save_escrow(seed, receive, &ctx.bumps)
    }

    /// Refunds the assets deposited in the escrow and closes the escrow account
    /// This function is callble only under conditions where the escrow agreement is not met,
    /// allowing the maker to reclaim their deposited assets- for example, if the taker does
    /// not fulfill their part of the agreement
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    /// Finalizes the escrow by transfering assets and closing the vault
    /// Only callable if the escrow conditions are fully met
    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }
}