use crate::{errors::CustomError, state::*};
use anchor_lang::prelude::*;
use anchor_lang::system_program;

pub fn initialize(
    ctx: Context<InitializeCurveConfiguration>,
    fees: f64,
) -> Result<()> {
    // Validate fee range
    if !(0.0..=100.0).contains(&fees) {
        return err!(CustomError::InvalidFee);
    }

    // Initialize configuration
    ctx.accounts.dex_configuration_account.set_inner(CurveConfiguration::new(fees));

    // Optional: Initialize global account with minimum rent if needed
    // Remove this if global account initialization is handled elsewhere
    if ctx.accounts.global_account.lamports() == 0 {
        let initial_funding = 10_000_000;
        **ctx.accounts.global_account.to_account_info().try_borrow_mut_lamports()? = initial_funding;
        **ctx.accounts.admin.to_account_info().try_borrow_mut_lamports()? -= initial_funding;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeCurveConfiguration<'info> {
    #[account(
        init,
        space = CurveConfiguration::ACCOUNT_SIZE,
        payer = admin,
        seeds = [CurveConfiguration::SEED.as_bytes()],
        bump,
    )]
    pub dex_configuration_account: Box<Account<'info, CurveConfiguration>>,

    /// CHECK
    #[account(
        mut,
        seeds = [b"global"],
        bump,
    )]
    pub global_account: AccountInfo<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
