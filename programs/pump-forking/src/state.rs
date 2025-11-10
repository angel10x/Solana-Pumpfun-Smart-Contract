use crate::consts::INITIAL_LAMPORTS_FOR_POOL;
use crate::consts::PROPORTION;
use crate::errors::CustomError;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

// Pre-calculated constants for optimization
const DECIMAL_SCALE: f64 = 1_000_000_000.0;
const DECIMAL_SCALE_SQUARED: f64 = 1_000_000.0 * 1_000_000_000.0;
const PROPORTION_F64: f64 = PROPORTION as f64;

#[account]
pub struct CurveConfiguration {
    pub fees: f64,
}

impl CurveConfiguration {
    pub const SEED: &'static str = "CurveConfiguration";

    // Discriminator (8) + f64 (8)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 8;

    pub fn new(fees: f64) -> Self {
        Self { fees }
    }
}

#[account]
pub struct LiquidityProvider {
    pub shares: u64, // The number of shares this provider holds in the liquidity pool ( didnt add to contract now )
}

impl LiquidityProvider {
    pub const SEED_PREFIX: &'static str = "LiqudityProvider"; // Prefix for generating PDAs

    // Discriminator (8) + f64 (8)
    pub const ACCOUNT_SIZE: usize = 8 + 8;
}

#[account]
pub struct LiquidityPool {
    pub creator: Pubkey,    // Public key of the pool creator
    pub token: Pubkey,      // Public key of the token in the liquidity pool
    pub total_supply: u64,  // Total supply of liquidity tokens
    pub reserve_token: u64, // Reserve amount of token in the pool
    pub reserve_sol: u64,   // Reserve amount of sol_token in the pool
    pub bump: u8,           // Nonce for the program-derived address
}

impl LiquidityPool {
    pub const POOL_SEED_PREFIX: &'static str = "liquidity_pool";
    pub const SOL_VAULT_PREFIX: &'static str = "liquidity_sol_vault";

    // Discriminator (8) + Pubkey (32) + Pubkey (32) + totalsupply (8)
    // + reserve one (8) + reserve two (8) + Bump (1)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1;

    // Constructor to initialize a LiquidityPool with two tokens and a bump for the PDA
    pub fn new(creator: Pubkey, token: Pubkey, bump: u8) -> Self {
        Self {
            creator,
            token,
            total_supply: 0_u64,
            reserve_token: 0_u64,
            reserve_sol: 0_u64,
            bump,
        }
    }
}

pub trait LiquidityPoolAccount<'info> {
    // Updates the token reserves in the liquidity pool
    fn update_reserves(&mut self, reserve_token: u64, reserve_sol: u64) -> Result<()>;

    // Allows adding liquidity by depositing an amount of two tokens and getting back pool shares
    fn add_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    // Allows removing liquidity by burning pool shares and receiving back a proportionate amount of tokens
    fn remove_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_account: &mut AccountInfo<'info>,
        authority: &Signer<'info>,
        bump: u8,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn buy(
        &mut self,
        // bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn sell(
        &mut self,
        // bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        amount: u64,
        bump: u8,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        to: &mut AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn transfer_sol_from_pool(
        &self,
        from: &mut AccountInfo<'info>,
        to: &Signer<'info>,
        amount: u64,
        bump: u8,
        system_program: &Program<'info, System>,
    ) -> Result<()>;
}

impl<'info> LiquidityPoolAccount<'info> for Account<'info, LiquidityPool> {
    fn update_reserves(&mut self, reserve_token: u64, reserve_sol: u64) -> Result<()> {
        self.reserve_token = reserve_token;
        self.reserve_sol = reserve_sol;
        Ok(())
    }

    fn add_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let token_supply = token_accounts.0.supply;
        
        // Transfer tokens to pool
        self.transfer_token_to_pool(
            token_accounts.2,
            token_accounts.1,
            token_supply,
            authority,
            token_program,
        )?;

        // Transfer SOL to pool
        self.transfer_sol_to_pool(
            authority,
            pool_sol_vault,
            INITIAL_LAMPORTS_FOR_POOL,
            system_program,
        )?;
        
        // Set total supply and update reserves
        self.total_supply = 1_000_000_000_000_000_000;
        self.update_reserves(token_supply, INITIAL_LAMPORTS_FOR_POOL)?;

        Ok(())
    }

    fn remove_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        authority: &Signer<'info>,
        bump: u8,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let token_amount = token_accounts.1.amount;
        self.transfer_token_from_pool(
            token_accounts.1,
            token_accounts.2,
            token_amount,
            token_program,
        )?;
        
        let sol_amount = pool_sol_vault.to_account_info().lamports();
        self.transfer_sol_from_pool(pool_sol_vault, authority, sol_amount, bump, system_program)?;

        Ok(())
    }

    fn buy(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        // Early validation
        if amount == 0 {
            return err!(CustomError::InvalidAmount);
        }

        // Optimized calculation: pre-compute sold tokens amount
        let sold_tokens: u64 = match self.total_supply.checked_sub(self.reserve_token) {
            Some(val) => val,
            None => return err!(CustomError::OverflowOrUnderflowOccurred),
        };
        
        // Convert to f64 with optimized scaling  
        let bought_amount = (sold_tokens as f64) / DECIMAL_SCALE_SQUARED;
        
        // Calculate amount out using bonding curve formula
        let amount_scaled = (amount as f64) / DECIMAL_SCALE;
        let bought_amount_squared = bought_amount * bought_amount;
        let root_val = (PROPORTION_F64 * amount_scaled + bought_amount_squared).sqrt();
        
        // Calculate tokens to receive
        let amount_out_f64 = (root_val - bought_amount) * DECIMAL_SCALE_SQUARED;
        let amount_out = amount_out_f64.round() as u64;

        // Validate sufficient tokens in reserve
        if amount_out > self.reserve_token {
            return err!(CustomError::NotEnoughTokenInVault);
        }

        // Update reserves with checked arithmetic
        self.reserve_sol = match self.reserve_sol.checked_add(amount) {
            Some(val) => val,
            None => return err!(CustomError::OverflowOrUnderflowOccurred),
        };
        
        self.reserve_token = match self.reserve_token.checked_sub(amount_out) {
            Some(val) => val,
            None => return err!(CustomError::OverflowOrUnderflowOccurred),
        };

        // Execute transfers
        self.transfer_sol_to_pool(authority, pool_sol_vault, amount, system_program)?;
        self.transfer_token_from_pool(
            token_accounts.1,
            token_accounts.2,
            amount_out,
            token_program,
        )?;
        
        Ok(())
    }

    fn sell(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        amount: u64,
        bump: u8,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        // Early validation
        if amount == 0 {
            return err!(CustomError::InvalidAmount);
        }

        // Validate sufficient tokens to sell
        if amount > self.reserve_token {
            return err!(CustomError::TokenAmountToSellTooBig);
        }

        // Optimized calculation: pre-compute sold tokens before and after
        let sold_tokens_before: u64 = match self.total_supply.checked_sub(self.reserve_token) {
            Some(val) => val,
            None => return err!(CustomError::OverflowOrUnderflowOccurred),
        };
        
        let reserve_token_after: u64 = match self.reserve_token.checked_add(amount) {
            Some(val) => val,
            None => return err!(CustomError::OverflowOrUnderflowOccurred),
        };
        
        let sold_tokens_after: u64 = match self.total_supply.checked_sub(reserve_token_after) {
            Some(val) => val,
            None => return err!(CustomError::OverflowOrUnderflowOccurred),
        };

        // Convert to f64 with optimized scaling
        let bought_amount = (sold_tokens_before as f64) / DECIMAL_SCALE_SQUARED;
        let result_amount = (sold_tokens_after as f64) / DECIMAL_SCALE_SQUARED;
        
        // Calculate SOL to receive using bonding curve formula
        let bought_amount_squared = bought_amount * bought_amount;
        let result_amount_squared = result_amount * result_amount;
        let amount_out_f64 = (bought_amount_squared - result_amount_squared) / PROPORTION_F64 * DECIMAL_SCALE;
        let amount_out = amount_out_f64.round() as u64;

        // Validate sufficient SOL in reserve
        if amount_out > self.reserve_sol {
            return err!(CustomError::NotEnoughSolInVault);
        }

        // Execute token transfer first (fail early if insufficient balance)
        self.transfer_token_to_pool(
            token_accounts.2,
            token_accounts.1,
            amount,
            authority,
            token_program,
        )?;

        // Update reserves with checked arithmetic
        self.reserve_token = reserve_token_after;
        self.reserve_sol = match self.reserve_sol.checked_sub(amount_out) {
            Some(val) => val,
            None => return err!(CustomError::OverflowOrUnderflowOccurred),
        };

        // Execute SOL transfer
        self.transfer_sol_from_pool(pool_sol_vault, authority, amount_out, bump, system_program)?;

        Ok(())
    }

    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: self.to_account_info(),
                },
                &[&[
                    LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
                    self.token.key().as_ref(),
                    &[self.bump],
                ]],
            ),
            amount,
        )?;
        Ok(())
    }

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    fn transfer_sol_from_pool(
        &self,
        from: &mut AccountInfo<'info>,
        to: &Signer<'info>,
        amount: u64,
        bump: u8,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.clone(),
                    to: to.to_account_info().clone(),
                },
                &[&[
                    LiquidityPool::SOL_VAULT_PREFIX.as_bytes(),
                    self.token.key().as_ref(),
                    &[bump],
                ]],
            ),
            amount,
        )?;
        Ok(())
    }

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        to: &mut AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }
}
