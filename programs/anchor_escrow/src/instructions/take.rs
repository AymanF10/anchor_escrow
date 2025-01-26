use anchor_lang::prelude::*;
use anchor_spl::{self, 
    associated_token::AssociatedToken, 
    token_interface::{TransferChecked, transfer_checked, CloseAccount, close_account, Mint, TokenAccount, TokenInterface}
    };
use crate::state::escrow;
use crate::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    pub maker: SystemAccount<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint =mint_a,
        associated_token::authority = taker,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint =mint_b,
        associated_token::authority = taker,
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker, 
        associated_token::mint =mint_b,
        associated_token::authority = maker,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker, 
        has_one = mint_b,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()], 
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        mut,
        associated_token::mint =mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,

}

impl<'info> Take<'info> {
    pub fn deposit(&mut self) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.token_program.to_account_info();
        let cpi_account: TransferChecked<'_> = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
            mint: self.mint_b.to_account_info(),

        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);
        transfer_checked(cpi_ctx, self.escrow.recieve, self.mint_b.decimals)?;
        Ok(())
    }
    
    pub fn withdraw(&mut self) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.token_program.to_account_info();
        let cpi_account: TransferChecked<'_> = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
            mint: self.mint_a.to_account_info(),

        };

        let signer_seeds: [&[&[u8]]; 1] = [&[ 
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, &signer_seeds);
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;
        Ok(())
    }
    pub fn close(&mut self) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.token_program.to_account_info();
        let cpi_account: CloseAccount<'_> = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority: self.escrow.to_account_info(),

        };

        let signer_seeds: [&[&[u8]]; 1] = [&[ 
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, &signer_seeds);
        close_account(cpi_ctx)?;
        Ok(())
    }
}