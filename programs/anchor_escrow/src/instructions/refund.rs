use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Refund<'info> {
    
    #[account(mut)] // The acount of the maker who initiates the refund   // Must a signer to authorize the refund
    pub maker: Signer<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,     // The mint of the token that was initially deposited into the escrow by the maker
    
    #[account(   // The maker's associated token account for Mint A, where tokens will be refunded to
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // The escrow account holding the state and terms of the escrow, including the seed and associated tokens
    // This account will be closed, and its remaining balance will be refunded to the maker
    #[account(
        mut,
        close = maker, 
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()], 
        bump = escrow.bump,
        constraint = (maker.key() == escrow.maker.key()),
    )]
    pub escrow: Account<'info, Escrow>,

    /// The vault account where the tokens from the maker were deposited and held during the escrow
    /// Tokens will be transferred back to the maker and account closed
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>, // SPL Associated Token Program used for handling operations on associated token accounts
    pub token_program: Interface<'info, TokenInterface>,  // SPL Token Program used for tokens transfers and other token operations
    pub system_program: Program<'info, System>,  // Solana System Program used for account creation, lamports transfer, etc.
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {

        let cpi_program: AccountInfo<'_> = self.token_program.to_account_info();
        // Set up the transfer checked call to move tokens from the vault back to the maker's ATA
        let cpi_account: TransferChecked<'_> = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // Execute the transfer checked operaiton, transfrering any remaining SOL to the maker
        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);
        let amount = self.vault.amount;
        transfer_checked(cpi_ctx, amount, self.mint_a.decimals)?;
        Ok(())
    }
    pub fn close_refund(&mut self) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.token_program.to_account_info();
        // Set up the closing of the vault account, transferring any remaining SOL to the maker
        let cpi_account: CloseAccount<'_> = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        // Prepare the signer seeds for authorizig operations with the escrow's PDA
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];
        // Execute the account closure using the signer seeds
        let cpi_ctx = CpiContext::new_with_signer(cpi_program,
            cpi_account,
            &signer_seeds,
        );
        close_account(cpi_ctx);
        Ok(())
    }
}