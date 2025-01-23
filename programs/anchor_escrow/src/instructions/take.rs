use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Escrow;
/// Defines the accounts needed for the `take` instruction, facilitating asset transfers and vault closure.

/// Defines the accounts needed for the `take` instruction, facilitating assets transfers and vault closure
#[derive(Accounts)]
pub struct Take<'info> {
    /// The participant initiating the `take` must be a signer
    #[account(mut)]
    pub taker: Signer<'info>,

    /// The original maker of the escrow, holds the counter assets
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    // Case scenario: Trading an NFT or SPL Token for an amount of stablecoin
    pub mint_a: InterfaceAccount<'info, Mint>,
    /// The mint of the token deposited by the maker into the escrow- e.g. an NFT or SPL Token
    pub mint_b: InterfaceAccount<'info, Mint>,
    /// The mint of the token expected to be received by the maker- e.g. a stablecoin

    /// Associated token account of the taker for receiving mint_a tokens
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Associated token account of the taker for depositing mint_b tokens to the maker
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Associated token account of the maker for receiving mint_b tokens from the taker
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The escrow account itself, holding state, terms and seeds
    #[account(
        mut,
        close = maker, // Allows the escrow account to be closed, and its remaining balance to be sent to maker once the escrow isn't needed anymore

        // Ensures the escrow account is linked to the specific maker, mint_a and mint_b
        // It ensures that the provided accounts match the ones specified on the creation of the escrow account
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    /// Vault for the assets deposited by the maker, controlled by the escrow logic
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// Represents the SPL Associated Token program used for managing token accounts, especially helpful for operations like creating and managing token accounts in a standardized way
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// The SPL Token program used for handling all token operations like transfers, minting, and burning within this transaction
    pub token_program: Interface<'info, TokenInterface>,

    /// The basic Solana system program used for fundamental operations like creating accounts and transferring lamports
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    /// Transfers the expected receive amount of mint_b from taker to the maker
    /// Represents the taker fulfilling their part of the escrow agreement
    pub fn deposit(&mut self) -> Result<()> {
        // Set up the acounts for transferring tokens with the SPL Token program
        let transfer_accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        // Creates a context for thhe Cross-Program Invocation (CPI) with the token program
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        // Execute the transfer checked operation to move th specified amount of mint_b tokens, ensuring that the token decimals are correctly handled
        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_b.decimals)
    }

    /// Withdraws the deposited mint_a tokens from the vault to the taker and closes the vault account
    /// This action finalizes the escrow by returning control of the deposited assets to the taker and cleaning up state
    pub fn withdraw_and_close_vault(&mut self) -> Result<()> {
        // Prepare the seeds for signing with the escrow's PDA
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // Set up tthe transfer of mint_a tokens from the vault back to the taker's ATA
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // Executes the transfer with signing authority from the PDA
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds,
        );
        transfer_checked(ctx, self.vault.amount, self.mint_a.decimals)?;

        // Set up the closure of the vault account, transferring any remaining SOL balance to the taker
        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // Executes the closure of the vault account
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds,
        );
        close_account(ctx)
    }
}