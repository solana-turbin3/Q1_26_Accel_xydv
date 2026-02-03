use anchor_lang::prelude::*;

use crate::state::whitelist::Whitelist;

// #[derive(Accounts)]
// pub struct WhitelistOperations<'info> {
//     #[account(
//         mut,
//         //address =
//     )]
//     pub admin: Signer<'info>,
//     #[account(
//         mut,
//         seeds = [b"whitelist"],
//         bump,
//     )]
//     pub whitelist: Account<'info, Whitelist>,
//     pub system_program: Program<'info, System>,
// }

// impl<'info> WhitelistOperations<'info> {
//     pub fn add_to_whitelist(&mut self, address: Pubkey) -> Result<()> {
//         if !self.whitelist.address.contains(&address) {
//             self.realloc_whitelist(true)?;
//             self.whitelist.address.push(address);
//         }
//         Ok(())
//     }

//     pub fn remove_from_whitelist(&mut self, address: Pubkey) -> Result<()> {
//         if let Some(pos) = self.whitelist.address.iter().position(|&x| x == address) {
//             self.whitelist.address.remove(pos);
//             self.realloc_whitelist(false)?;
//         }
//         Ok(())
//     }

//     pub fn realloc_whitelist(&self, is_adding: bool) -> Result<()> {
//         // Get the account info for the whitelist
//         let account_info = self.whitelist.to_account_info();

//         if is_adding {
//             // Adding to whitelist
//             let new_account_size = account_info.data_len() + std::mem::size_of::<Pubkey>();
//             // Calculate rent required for the new account size
//             let lamports_required = (Rent::get()?).minimum_balance(new_account_size);
//             // Determine additional rent required
//             let rent_diff = lamports_required - account_info.lamports();

//             // Perform transfer of additional rent
//             let cpi_program = self.system_program.to_account_info();
//             let cpi_accounts = system_program::Transfer {
//                 from: self.admin.to_account_info(),
//                 to: account_info.clone(),
//             };
//             let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
//             system_program::transfer(cpi_context, rent_diff)?;

//             // Reallocate the account
//             account_info.resize(new_account_size)?;
//             msg!("Account Size Updated: {}", account_info.data_len());
//         } else {
//             // Removing from whitelist
//             let new_account_size = account_info.data_len() - std::mem::size_of::<Pubkey>();
//             // Calculate rent required for the new account size
//             let lamports_required = (Rent::get()?).minimum_balance(new_account_size);
//             // Determine additional rent to be refunded
//             let rent_diff = account_info.lamports() - lamports_required;

//             // Reallocate the account
//             account_info.resize(new_account_size)?;
//             msg!("Account Size Downgraded: {}", account_info.data_len());

//             // Perform transfer to refund additional rent
//             **self.admin.to_account_info().try_borrow_mut_lamports()? += rent_diff;
//             **self.whitelist.to_account_info().try_borrow_mut_lamports()? -= rent_diff;
//         }

//         Ok(())
//     }
// }

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Whitelist::DISCRIMINATOR.len() + Whitelist::INIT_SPACE,
        seeds = [b"whitelist", address.key().as_ref()],
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(&mut self, address: Pubkey, bumps: &AddToWhitelistBumps) -> Result<()> {
        self.whitelist.set_inner(Whitelist {
            address,
            bump: bumps.whitelist,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        close = admin,
        seeds = [b"whitelist", address.key().as_ref()],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist(&mut self, _address: Pubkey) -> Result<()> {
        Ok(())
    }
}
