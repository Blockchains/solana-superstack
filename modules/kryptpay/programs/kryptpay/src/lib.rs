use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{transfer, Mint, Token, TokenAccount, Transfer as SplTransfer}
    },
    solana_program::system_instruction,
};
// use crate::instructions::SwapExactTokensForTokens;
// mod instructions;
// mod constants;
// mod errors;
// mod state;

declare_id!("6k73LWhMtLhJLVyC3qQGa8pvDZ2GUXnLAb9n3juk8u3A");

#[program]
pub mod kryptpay {
    use super::*;

    pub fn transfer_spl_tokens(ctx: Context<TransferSpl>, amount: u64) -> Result<()> {
        // Deduct platform fee (0.05% of the lamports) before transferring
        let platform_fee = amount * 5 / 10000;
        let reward_amount = 10*10^6;

        let amount_after_fee = amount - platform_fee;

        let destination = &ctx.accounts.to_ata;
        let source = &ctx.accounts.from_ata;
        let token_program = &ctx.accounts.token_program;
        let authority = &ctx.accounts.from;

        // Transfer tokens from taker to initializer
        let cpi_accounts = SplTransfer {
            from: source.to_account_info().clone(),
            to: destination.to_account_info().clone(),
            authority: authority.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();
        
        transfer(
            CpiContext::new(cpi_program, cpi_accounts),
            amount_after_fee)?;

        // Transfer platform fee to the platform account
        let platform_account = ctx.accounts.platform_account.to_account_info().clone();
        transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), SplTransfer {
                from: ctx.accounts.from_ata.to_account_info().clone(),
                to: platform_account,
                authority: ctx.accounts.from.to_account_info().clone(),
            }),
            platform_fee,
        )?;

        // Reward user with 10 Krystal tokens
        let krystal_token_account = ctx.accounts.krystal_token_account.to_account_info().clone();
        transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), SplTransfer {
                from: krystal_token_account,
                to: destination.to_account_info().clone(),
                authority: ctx.accounts.from.to_account_info().clone(),
            }),
            reward_amount,
        )?;

        Ok(())
    }

    pub fn transfer_lamports(ctx: Context<TransferLamports>, amount: u64) -> Result<()> {
        // Calculate the platform fee (0.05% of the transaction amount)
        let platform_fee = amount * 5 / 10000;

        // Calculate the amount to transfer after deducting the platform fee
        let amount_after_fee = amount - platform_fee;

        let from_account = &ctx.accounts.from;
        let to_account = &ctx.accounts.to;
        let reward_amount = 10*10^6;

        let destination = &ctx.accounts.to_ata;
        let source = &ctx.accounts.krystal_token_account;
        let token_program = &ctx.accounts.token_program;
        let authority = &ctx.accounts.from;

        // Create the transfer instruction
        let transfer_instruction = system_instruction::transfer(
            from_account.key, 
            to_account.key, 
            amount_after_fee
        );

        // Invoke the transfer instruction
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                from_account.to_account_info(),
                to_account.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;

        // Transfer platform fee to the platform account
        let platform_account = ctx.accounts.platform_account.to_account_info().clone();
        let transfer_platform_instruction = system_instruction::transfer(
            ctx.accounts.from.key,
            platform_account.key,
            platform_fee,
        );
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_platform_instruction,
            &[
                ctx.accounts.from.to_account_info(),
                platform_account,
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;

        // Reward user with 10 Krystal tokens
        // Transfer tokens from taker to initializer
        let cpi_accounts = SplTransfer {
            from: source.to_account_info().clone(),
            to: destination.to_account_info().clone(),
            authority: authority.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();
        
        transfer(
            CpiContext::new(cpi_program, cpi_accounts),
            reward_amount)?;
        Ok(())
    }

    // pub fn swap_tokens(
    //     ctx: Context<SwapExactTokensForTokens>,
    //     swap_a: bool,
    //     input_amount: u64,
    //     min_output_amount: u64,
    // ) -> Result<()> {
    //     instructions::swap_exact_tokens_for_tokens(ctx, swap_a, input_amount, min_output_amount) 
    // }

}

#[derive(Accounts)]
pub struct TransferLamports<'info> {
    #[account(mut)]
    pub from: Signer<'info>,

    #[account(mut)]
    /// CHECK: The `to` field is marked as mutable because it represents the account
    ///         to which lamports are being transferred. Since the transfer operation
    ///         requires modifying the account data, it is necessary for this field
    ///         to be mutable. No additional checks through types are necessary for
    ///         this field as the safety of the operation is ensured by the program logic.
    pub to: AccountInfo<'info>,
    pub krystal_token_account: Account<'info, TokenAccount>,
    pub to_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    /// CHECK: The `to` field is marked as mutable because it represents the account
    ///         to which lamports are being transferred. Since the transfer operation
    ///         requires modifying the account data, it is necessary for this field
    ///         to be mutable. No additional checks through types are necessary for
    ///         this field as the safety of the operation is ensured by the program logic.
    pub platform_account: AccountInfo<'info>,
    // pub krystal_token: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TransferSpl<'info> {
    pub from: Signer<'info>,
    #[account(mut)]
    pub from_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub krystal_token: Program<'info, Token>,
    pub krystal_token_account: Account<'info, TokenAccount>,
    /// CHECK: The `to` field is marked as mutable because it represents the account
    ///         to which lamports are being transferred. Since the transfer operation
    ///         requires modifying the account data, it is necessary for this field
    ///         to be mutable. No additional checks through types are necessary for
    ///         this field as the safety of the operation is ensured by the program logic.
    pub platform_account: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

