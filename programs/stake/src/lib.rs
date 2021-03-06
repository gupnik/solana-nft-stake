use anchor_lang::{AccountsExit, Key, prelude::*};
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer};
use spl_token::state::Account as SPLTokenAccount;
use solana_program::{program::invoke, program_error::ProgramError, program_pack::Pack, system_instruction, system_program};

declare_id!("5JqFVkV8QNpm7WpiUkRxrXbDVncRYgdbia8EWDWNrQ62");

const PREFIX: &str = "stake";
#[program]
pub mod stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> ProgramResult {
        let stake_account = &mut ctx.accounts.stake_account;
        stake_account.bump = bump;
        stake_account.authority = *ctx.accounts.authority.to_account_info().key;

        Ok(())
    }

    pub fn stake_jambo(ctx: Context<StakeJambo>, bump: u8) -> ProgramResult {
        let stake_jambo_account = &mut ctx.accounts.stake_jambo_account;
        stake_jambo_account.bump = bump;
        stake_jambo_account.jambo_mint = *ctx.accounts.jambo_mint.to_account_info().key;
        stake_jambo_account.jambo_mint_pool = *ctx.accounts.jambo_mint_pool.to_account_info().key;

        // Transfer the underlying assets to the underlying assets pool
        let cpi_accounts = Transfer {
            from: ctx.accounts.jambo_mint_src.to_account_info(),
            to: ctx.accounts.jambo_mint_pool.to_account_info(),
            authority: ctx.accounts.authority.clone(),
        };
        let cpi_token_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_token_program, cpi_accounts);
        let underlying_transfer_amount = 1;
        token::transfer(cpi_ctx, underlying_transfer_amount)?;

        Ok(())
    }

    #[access_control(UnstakeJambo::accounts(&ctx))]
    pub fn unstake_jambo(ctx: Context<UnstakeJambo>) -> ProgramResult {
        let stake_jambo_account = &ctx.accounts.stake_jambo_account;
        let seeds = &[
            PREFIX.as_bytes(),
            stake_jambo_account.jambo_mint.as_ref(),
            &[stake_jambo_account.bump]
        ];
        let signer = &[&seeds[..]];

        // Transfer the underlying assets to the underlying assets pool
        let cpi_accounts = Transfer {
            from: ctx.accounts.jambo_mint_pool.to_account_info(),
            to: ctx.accounts.jambo_mint_dest.to_account_info(),
            authority: ctx.accounts.stake_jambo_account.to_account_info(),
        };
        let cpi_token_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_token_program, cpi_accounts, signer);
        let underlying_transfer_amount = 1;
        token::transfer(cpi_ctx, underlying_transfer_amount)?;

        Ok(())
    }

    #[access_control(RestakeJambo::accounts(&ctx))]
    pub fn restake_jambo(ctx: Context<RestakeJambo>) -> ProgramResult {
        let stake_jambo_account = &ctx.accounts.stake_jambo_account;

        // Transfer the underlying assets to the underlying assets pool
        let cpi_accounts = Transfer {
            from: ctx.accounts.jambo_mint_src.to_account_info(),
            to: ctx.accounts.jambo_mint_pool.to_account_info(),
            authority: ctx.accounts.authority.clone(),
        };
        let cpi_token_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_token_program, cpi_accounts);
        let underlying_transfer_amount = 1;
        token::transfer(cpi_ctx, underlying_transfer_amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {   
    #[account(init, seeds=[PREFIX.as_bytes(), authority.key().as_ref()], bump=bump, payer=authority)] 
    pub stake_account: Account<'info, StakeAccount>,
    // // #[account(signer, constraint= authority.data_is_empty() && authority.lamports() > 0)]
    // authority: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // // #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    // rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct StakeJambo<'info> {
    #[account(init, seeds=[PREFIX.as_bytes(), jambo_mint.key().as_ref()], bump=bump, payer=authority)] 
    pub stake_jambo_account: Account<'info, StakeJamboAccount>,
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    pub jambo_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub jambo_mint_src: Box<Account<'info, TokenAccount>>,
    #[account(init,
        seeds = [&stake_jambo_account.key().to_bytes()[..], b"jamboMintPool"],
        bump,
        payer = authority,    
        token::mint = jambo_mint,
        token::authority = stake_jambo_account,
    )]
    pub jambo_mint_pool: Box<Account<'info, TokenAccount>>,

    pub token_program: AccountInfo<'info>,
    // pub associated_token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnstakeJambo<'info> {
    #[account(
        mut,
        seeds = [PREFIX.as_bytes(), stake_jambo_account.jambo_mint.key().as_ref()],
        bump = stake_jambo_account.bump
    )]
    pub stake_jambo_account: Account<'info, StakeJamboAccount>,
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    pub jambo_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub jambo_mint_dest: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub jambo_mint_pool: Box<Account<'info, TokenAccount>>,

    pub token_program: AccountInfo<'info>,
    // pub associated_token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
impl<'info> UnstakeJambo<'info> {
    fn accounts(ctx: &Context<UnstakeJambo<'info>>) -> ProgramResult {
        // Validate the mint pool is the same as on the StakeJamboAccount
        if *ctx.accounts.jambo_mint_pool.to_account_info().key != ctx.accounts.stake_jambo_account.jambo_mint_pool {
            return Err(ErrorCode::JamboMintPoolAccountDoesNotMatchStake.into())
        }

        // Validate the jambo mint is the same as on the StakeJamboAccount
        if *ctx.accounts.jambo_mint.to_account_info().key != ctx.accounts.stake_jambo_account.jambo_mint {
            return Err(ErrorCode::JamboMintDoesNotMatchStake.into())
        }

        Ok(())
    }
}


#[derive(Accounts)]
pub struct RestakeJambo<'info> {
    #[account(
        mut,
        seeds = [PREFIX.as_bytes(), stake_jambo_account.jambo_mint.key().as_ref()],
        bump = stake_jambo_account.bump
    )]
    pub stake_jambo_account: Account<'info, StakeJamboAccount>,
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    pub jambo_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub jambo_mint_src: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub jambo_mint_pool: Box<Account<'info, TokenAccount>>,

    pub token_program: AccountInfo<'info>,
    // pub associated_token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
impl<'info> RestakeJambo<'info> {
    fn accounts(ctx: &Context<RestakeJambo<'info>>) -> ProgramResult {
        // Validate the mint pool is the same as on the StakeJamboAccount
        if *ctx.accounts.jambo_mint_pool.to_account_info().key != ctx.accounts.stake_jambo_account.jambo_mint_pool {
            return Err(ErrorCode::JamboMintPoolAccountDoesNotMatchStake.into())
        }

        // Validate the jambo mint is the same as on the StakeJamboAccount
        if *ctx.accounts.jambo_mint.to_account_info().key != ctx.accounts.stake_jambo_account.jambo_mint {
            return Err(ErrorCode::JamboMintDoesNotMatchStake.into())
        }

        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct StakeJamboAccount {
    pub jambo_mint: Pubkey,
    pub jambo_mint_pool: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(Default)]
pub struct StakeAccount {
    pub authority: Pubkey,
    pub bump: u8,
}

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)] 
// pub struct StakeJamboAccountData {
//     pub jambo_mint: Pubkey,
// }

#[error]
pub enum ErrorCode {
    #[msg("Mint pool is not same as on the StakeJamboAccount!")]
    JamboMintPoolAccountDoesNotMatchStake,
    #[msg("Jambo mint is not same as on the StakeJamboAccount!")]
    JamboMintDoesNotMatchStake,
}