use anchor_lang::prelude::*;

declare_id!("73bhLRuwcNwd7prQnWqGrSARy5ozmKykGEUBPzwCSGgS");

pub mod state;
use state::*;
pub mod instructions;
use instructions::*;

#[program]
pub mod anchor_escrow {
    use super::*;
    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.deposit(deposit);
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)
    } 
    
    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw()?;
        ctx.accounts.close()?;
        Ok(())
    }
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund()?;
        ctx.accounts.close_refund()?;
        Ok(())
    }   
}
