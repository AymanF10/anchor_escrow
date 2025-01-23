use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::Escrow;

/// Defines the accounts needed to execute the `make` instruction, including the maker, token mints, token acocunts and system programs
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    /// The user initiating the escrow who signs the transaction
    /// The signer must be this user, approving the transaction's terms and authorizing the transfer of funds
    #[account(mut)]
    pub maker: Signer<'info>,

    /// Represents the token type (mint) that the maker will deposit into the escrow
    /// This account stores information about the specific token type, such as total supply and minting authority
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    /// Represents the token type that the maker expects to receive from the escrow
    /// This is used to verify the type of tokens the escrow will handle in the transaction opposite to `mint_a`
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    /// The maker's token account for `mint_a`
    /// This is where the tokens that will be deposited into the escrow are initially held
    #[account(
        mut, // This account's balance can be modified- decremented
        associated_token::mint = mint_a, // Links this account to the `mint_a` token type
        associated_token::authority = maker, // Confirms the maker controls this account
        associated_token::token_program = token_program // Specifies the token management program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    /// The actual escrow account that will hold the state of the escrow transaction, including data like the seed, amounts to be sent/received, and ownership details
    #[account(
        init, // Indicates this account will be created with this transaction if it doesn't already exist
        payer = maker, // Specifies that the maker will pay for the account creation
        space = 8 + Escrow::INIT_SPACE, // Defines how much data storage space is needed

        // seeds and bump provide a mechanism for creating a predictable, yet secure, address for this account using a derived address
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    /// A special token account created to hold the `mint_a` tokens deposited by the maker. This account is controlled by the escrow, and acts as the lockbox for the assets until conditions are met
    #[account(
        init, // To create the account during this transaction
        payer = maker, // Indicates the maker is paying for the setup
        associated_token::mint = mint_a, // Ensures this vault can only hold the type of tokens specified by the `mint_a`
        associated_token::authority = escrow, // Transfer control of this account to the escrow program, meaning only the escrow can authorize transactions from it
        associated_token::token_program = token_program // Specifies the token management program 
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// Represents the SPL Associated Token program used for managing token accounts, especially helpful for operations like creating and managing token accounts in a standardized way
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// The SPL Token program used for handling all token operations like transfers, minting, and burning within this transaction
    pub token_program: Interface<'info, TokenInterface>,

    /// The basic Solana system program used for fundamental operations like creating accounts and transferring lamports
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    /// This function is designed to initialize or update the escrow account with necessary parameters to establish the conditions under which the escrow operates
    pub fn save_escrow(&mut self, seed: u64, receive: u64, bumps: &MakeBumps) -> Result<()> {
        // Sets the inner state of the `escrow` accpimt with the new `Escrow` struct, passing in values such as the unique `seed`, identifies of the token types (`mint_a`, `mint_b`), and the amount the maker expects to receive (`receive`)
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive,
            bump: bumps.escrow, // The bump seed is included to ensure that the address of the escrow account is derived securely and predictably using the provided seeds
        });
        Ok(())
    }

    /// This function handles the acutal transfer of tokens fom the maker's account to the escrow's vault. It ensures that the tokens are safely locked until the escrow conditions are met
    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        // TranferChecked is created specifying the accounts involved in the transfer- from the maker's ata to the escrow's vault
        let transfer_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(), // Uses the token mint information to ensure the transfer respects the token's properties (e.g. decimals)
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        // Context is set up with the `token_program`, allowing the escrow program to call the SPL Token program's `transfer_checked` function securely
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        // The `transfer_checked` function is invoked to move `deposit` amount of tokens, validated by the token's decimal specification to ensure accuracy and correctness
        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals)
    }
}