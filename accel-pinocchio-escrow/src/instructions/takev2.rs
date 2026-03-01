use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::TokenAccount,
};

use crate::{state::Escrow, ID};

pub fn process_takev2_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [taker, maker, mint_a, mint_b, escrow_account, escrow_ata_a, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }
    log!("1");
    let (amount_to_receive, amount_to_give, bump) = {
        let escrow_data = unsafe { escrow_account.borrow_unchecked() };
        let escrow_state = wincode::deserialize::<Escrow>(escrow_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        if escrow_state.maker != *maker.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }

        (
            escrow_state.amount_to_receive,
            escrow_state.amount_to_give,
            escrow_state.bump,
        )
    };
    log!("2");

    {
        let maker_ata_b_state = TokenAccount::from_account_view(maker_ata_b)?;
        if maker_ata_b_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if *maker_ata_b_state.mint().as_array() != *mint_b.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    log!("3");

    {
        let taker_ata_a_state = TokenAccount::from_account_view(taker_ata_a)?;
        if taker_ata_a_state.owner() != taker.address() {
            return Err(ProgramError::IllegalOwner);
        }
    }
    log!("4");

    let bump = [bump.to_le()];

    // let seeds: [&[u8]; 3] = [b"escrow", maker.address().as_array(), &bump];
    // let expected_escrow = derive_address(&seeds, None, ID.as_array());

    // if escrow_account.address().as_array() != &expected_escrow {
    //     return Err(ProgramError::InvalidAccountData);
    // }

    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];

    log!("5");
    Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: u64::from_le_bytes(amount_to_receive),
    }
    .invoke()?;
    log!("6");

    Transfer {
        from: escrow_ata_a,
        to: taker_ata_a,
        authority: escrow_account,
        amount: u64::from_le_bytes(amount_to_give),
    }
    .invoke_signed(&[Signer::from(&seed)])?;
    log!("7");

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
