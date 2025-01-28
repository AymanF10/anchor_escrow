use anchor_lang::prelude::*;
use anchor_spl::{ 
    associated_token::AssociatedToken, 
    token_interface::{TransferChecked, transfer_checked, Mint, TokenAccount, TokenInterface}
    };
use crate::state::escrow;
use crate::Escrow;


#[derive(Accounts)]
#[instruction(seed: u64)] // this allows a user to have more than 1 escrow otherwise it will throw error if user try to create another escrow
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,  // The user initiating the escrow who signs the transaction. The signer must be this user, approving the transaction's terms and authorizing the transfer of funds
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init,
        payer = maker,
        space = 8 + Escrow:: INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()], // escrow is a word we give, we can use anyother word too
        bump,
    )]

    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>, // this program owns associated token account
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>
}

impl<'info> Make<'info> {
    pub fn init_escrow(&mut self, seed: u64, recieve: u64, bumps: &MakeBumps) -> Result<()> { // This function is designed to initialize or update the escrow account with necessary parameters to establish the conditions under which the escrow operates

        self.escrow.set_inner(Escrow { 
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            recieve,
            bump: bumps.escrow,  // The bump seed is included to ensure that the address of the escrow account is derived securely 
        });
        Ok(())
    }

    pub fn deposit(&mut self, deposit: u64) {
        let cpi_program: AccountInfo<'_> = self.token_program.to_account_info();
        let cpi_account: TransferChecked<'_> = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
            mint: self.mint_a.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);
        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals);
        
    }

}