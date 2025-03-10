use anchor_lang::prelude::*;

pub mod account;
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod states;
pub mod utils;

use instructions::*;
use states::Configuration;

declare_id!("BDeQaWDdyQoGDWfvNrrc2ovCKCoxrRyQHZsDAiHuAuHV");

#[program]
pub mod pumpfun_forking {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, config: Configuration) -> Result<()> {
        ctx.accounts.process(config)
    }

    pub fn create_pool(
        ctx: Context<CreatePool>,
        fee_lamports: u64,
        token_amount: u64,
        raydium_token_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .process(fee_lamports, token_amount, raydium_token_amount)
    }

    pub fn buy(ctx: Context<Buy>, in_amount: u64) -> Result<()> {
        ctx.accounts.process(in_amount, ctx.bumps.sol_pool)
    }

    pub fn sell(ctx: Context<Sell>, in_amount: u64) -> Result<()> {
        ctx.accounts.process(in_amount, ctx.bumps.sol_pool)
    }

    /// Initiazlize a swap pool
    pub fn raydium_integrate(ctx: Context<RaydiumIntegrate>, nonce: u8) -> Result<()> {
        let opentime = Clock::get()?.unix_timestamp as u64;
        instructions::initialize(ctx, nonce, opentime)
    }
}
