use anchor_lang::prelude::*;

pub mod consts;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;
use state::CurveConfiguration;

declare_id!("BDeQaWDdyQoGDWfvNrrc2ovCKCoxrRyQHZsDAiHuAuHV");

#[program]
pub mod pumpfun_forking {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fees: f64) -> Result<()> {
        instructions::initialize(ctx, fees)
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
        instructions::buy(ctx, in_amount)
    }

    pub fn sell(ctx: Context<Sell>, in_amount: u64) -> Result<()> {
        instructions::sell(ctx, in_amount)
    }

    /// Initiazlize a swap pool
    pub fn raydium_migrate(ctx: Context<RaydiumMigrate>, nonce: u8, open_time: u64) -> Result<()> {
        ctx.accounts.process(nonce, open_time)
    }
}
