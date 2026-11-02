use anchor_lang::prelude::*;
use anchor_lang::system_program;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("DLpi4zr3TmBoR168KVi9xN2kw1b4WsP6DoKqzB4LSCR8");

#[program]
mod hello_anchor {
    use super::*;

    pub fn init_analyst(ctx: Context<InitAnalyst>) -> Result<()> {
        let analyst = &ctx.accounts.analyst;
        Ok(())
    }

    pub fn create_insight(
        ctx: Context<CreateInsight>,
        authority: Pubkey,
        crypto_name: String,
        encrypted_content: String,
        timestamp: i64,
    ) -> Result<()> {
        let insight = &mut ctx.accounts.insight;
        insight.crypto_name = crypto_name;
        insight.encrypted_content = encrypted_content;
        insight.timestamp = timestamp;
        insight.validated = false;
        Ok(())
    }

    pub fn participate(ctx: Context<Participate>, participant_public_key: Pubkey) -> Result<()> {
        // Assuming the actual decryption and verification to happen client-side

        Ok(())
    }

    pub fn validate_insight(
        ctx: Context<ValidateInsight>,
        real_data: String,
        post_id: u32,
    ) -> Result<()> {
        let insight = &mut ctx.accounts.insight;
        let analyst = &mut ctx.accounts.analyst;
        /* if is_valid {
            analyst.trust_points += 1; // Increment trust points for valid insights
        } else {
            analyst.trust_points = analyst.trust_points.saturating_sub(1); // Decrement trust points carefully to avoid underflow
        } */
        insight.validated = true;
        Ok(())
    }

    

    pub fn buy_post(ctx: Context<BuyPost>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitAnalyst<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = 1000)]
    analyst: Account<'info, Analyst>,
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreateInsight<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = 1000)]
    pub insight: Account<'info, Insight>,
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ValidateInsight<'info> {
    #[account(mut, has_one = analyst)]
    pub insight: Account<'info, Insight>,

    #[account(mut)]
    pub analyst: Account<'info, Analyst>,
}



#[derive(Accounts)]
pub struct BuyPost {}

#[derive(Accounts)]
pub struct Participate<'info> {
    #[account(mut)]
    pub participant: Signer<'info>,

    #[account(mut)]
    pub insight: Account<'info, Insight>,
    pub analyst: Account<'info, Analyst>,

    #[account(init, payer = participant, space = 1000)]
    pub participant_to_insight: Account<'info, ParticipantToInsight>,
    pub system_program: AccountInfo<'info>,


}

#[account]
pub struct Analyst {
    pub_key: Pubkey,
    trust_point: u8,
    insight_count: u8,
}

impl Analyst {
    pub fn init_auth(&mut self) {
        self.insight_count = 0;
        self.trust_point = 0;
    }
}

#[account]
pub struct Insight {
    pub insight_id: u32,
    pub analyst: Pubkey,
    pub encrypted_content: String,
    pub crypto_name: String,
    pub timestamp: i64,
    pub validated: bool,
}

impl Insight {
    pub fn init(&mut self) {
        self.validated = false;
        let clock = Clock::get().unwrap();
        self.timestamp = clock.unix_timestamp;
    }
}

#[account]
pub struct ParticipantToInsight {
    participant: Pubkey,
    analyst_pubkey: Pubkey,
    encrypted_content: String,
}