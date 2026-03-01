use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::TokenAccount,
};

use crate::{state::Escrow, ID};

pub fn process_take_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [taker, maker, mint_a, mint_b, escrow_account, escrow_ata_a, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    let (amount_to_receive, amount_to_give, bump) = {
        let escrow_state = Escrow::from_account_info(escrow_account)?;

        if *escrow_state.maker().as_array() != *maker.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }

        (
            escrow_state.amount_to_receive(),
            escrow_state.amount_to_give(),
            escrow_state.bump,
        )
    };

    {
        let maker_ata_b_state = TokenAccount::from_account_view(maker_ata_b)?;
        if maker_ata_b_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if *maker_ata_b_state.mint().as_array() != *mint_b.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    {
        let taker_ata_a_state = TokenAccount::from_account_view(taker_ata_a)?;
        if taker_ata_a_state.owner() != taker.address() {
            return Err(ProgramError::IllegalOwner);
        }
    }

    let bump = [bump.to_le()];

    let seeds: [&[u8]; 3] = [b"escrow", maker.address().as_array(), &bump];
    let expected_escrow = derive_address(&seeds, None, ID.as_array());

    if escrow_account.address().as_array() != &expected_escrow {
        return Err(ProgramError::InvalidAccountData);
    }

    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];

    Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: amount_to_receive,
    }
    .invoke()?;

    Transfer {
        from: escrow_ata_a,
        to: taker_ata_a,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(&[Signer::from(&seed)])?;

    CloseAccount {
        account: escrow_ata_a,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&[Signer::from(&seed)])?;

    // close the pda by setting lamports = 0
    let lamports = escrow_account.lamports();
    maker.set_lamports(maker.lamports() + lamports);
    escrow_account.set_lamports(0);
    escrow_account.resize(0)?;

    Ok(())
}
