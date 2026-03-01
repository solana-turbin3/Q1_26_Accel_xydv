use crate::state::Escrow;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};

pub fn process_refund_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [maker, escrow_account, maker_ata_a, escrow_ata, _token_program, _system_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    let (amount_to_give, bump) = {
        let escrow_state = Escrow::from_account_info(escrow_account)?;

        if *escrow_state.maker().as_array() != *maker.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }

        (escrow_state.amount_to_give(), escrow_state.bump)
    };

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];

    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: maker_ata_a,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(&[Signer::from(&seed)])?;

    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&[Signer::from(&seed)])?;

    Ok(())
}
