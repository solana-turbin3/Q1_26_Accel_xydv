use anchor_lang::prelude::*;

declare_id!("DLEfvPQGYZiYKAuFyd6iJ5RKnkMqioDhkmcq4NzSiRfM");

#[program]
pub mod magicblock_er {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
