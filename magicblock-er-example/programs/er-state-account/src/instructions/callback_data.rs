use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct CallbackData<'info> {
    /// This check ensure that the vrf_program_identity (which is a PDA) is a singer
    /// enforcing the callback is executed by the VRF program trough CPI
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, UserAccount>,
}

impl<'info> CallbackData<'info> {
    pub fn callback_data(&mut self, randomness: [u8; 32]) -> Result<()> {
        let random_data = ephemeral_vrf_sdk::rnd::random_u64(&randomness);
        msg!("consuming random data: {:?}", random_data);
        self.user.data = random_data;
        Ok(())
    }
}
