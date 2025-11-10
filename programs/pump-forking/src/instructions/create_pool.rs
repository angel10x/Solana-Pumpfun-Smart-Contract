use anchor_lang::{
  prelude::*,
  solana_program::system_instruction::{self},
};
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{Mint, Token, TokenAccount},
};
use core::mem::size_of;

#[derive(Accounts)]
pub struct CreatePool<'info> {
  #[account(
      seeds = [ GLOBAL_INFO_SEED ],
      bump
  )]
  pub global_info: Account<'info, GlobalInfo>,

  /// CHECK: This is a PDA designated for fee allocation.
  #[account(
      seeds = [ PUMP_FUN_FEE_SEED ],
      bump
  )]
  pub fee_account: AccountInfo<'info>,

  #[account(
      init,
      payer = payer,
      seeds =[ &token_mint.key().to_bytes(), BONDING_CURVE_SEED ],
      space = 8 + size_of::<BondingCurve>(),
      bump
  )]
  pub bonding_curve: Account<'info, BondingCurve>,

  pub token_mint: Account<'info, Mint>,

  #[account(
      mut,
      associated_token::mint = token_mint,
      associated_token::authority = payer
  )]
  pub user_ata: Account<'info, TokenAccount>,

  /// CHECK:
  #[account(
      mut,
      seeds = [ &token_mint.key().to_bytes() , SOL_POOL_SEED ],
      bump
  )]
  pub sol_pool: AccountInfo<'info>,

  #[account(
      associated_token::mint = token_mint,
      associated_token::authority = sol_pool
  )]
  pub token_pool: Account<'info, TokenAccount>,

  #[account(mut)]
  pub payer: Signer<'info>,

  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

impl<'info> CreatePool<'info> {
  pub fn process(
      &mut self,
      fee_lamports: u64,
      token_amount: u64,
      raydium_token_amount: u64,
  ) -> Result<()> {
      msg!("Calling create_pool...");
      msg!(
          "Sent Create Fee to Fee Wallet : {} Sol ",
          ((fee_lamports as f32) / (1_000_000_000 as f32))
      );
      let fee_account = &self.fee_account;

      let transfer_instruction = system_instruction::transfer(
          &self.payer.to_account_info().key(),
          &fee_account.key(),
          fee_lamports,
      );

      anchor_lang::solana_program::program::invoke_signed(
          &transfer_instruction,
          &[
              self.payer.to_account_info(),
              fee_account.to_account_info(),
              self.system_program.to_account_info(),
          ],
          &[],
      )?;

      self.bonding_curve.token_total_supply = self.token_mint.supply;
      self.bonding_curve.raydium_token = raydium_token_amount;
      self.bonding_curve.virtual_sol_reserves = self.global_info.config.initial_virtual_sol;
      self.bonding_curve.virtual_token_reserves = token_amount;
      self.bonding_curve.real_sol_reserves = 0;
      self.bonding_curve.real_token_reserves = token_amount;

      Ok(())
  }
}
