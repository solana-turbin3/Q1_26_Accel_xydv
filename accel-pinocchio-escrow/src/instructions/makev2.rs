use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use wincode::SchemaRead;

use crate::state::Escrow;

#[derive(SchemaRead)]
pub struct MakeV2InstructionData {
    pub amount_to_receive: u64,
    pub amount_to_give: u64,
    pub bump: u8,
}

pub fn process_makev2_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, escrow_account, maker_ata, escrow_ata, system_program, token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let ix_data = wincode::deserialize::<MakeV2InstructionData>(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let (amount_to_give, amount_to_receive, bump) = (
        ix_data.amount_to_give,
        ix_data.amount_to_receive,
        ix_data.bump,
    );

    {
        let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(&maker_ata)?;
        if maker_ata_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if maker_ata_state.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow_account_pda, *escrow_account.address().as_array());

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    CreateAccount {
        from: maker,
        to: escrow_account,
        lamports: Rent::get()?.try_minimum_balance(Escrow::LEN)?,
        space: Escrow::LEN as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[seeds.clone()])?;

    let escrow_state = Escrow::from_account_info(escrow_account)?;

    escrow_state.maker = *maker.address().as_array();
    escrow_state.mint_a = *mint_a.address().as_array();
    escrow_state.mint_b = *mint_b.address().as_array();
    escrow_state.amount_to_receive = amount_to_receive.to_le_bytes();
    escrow_state.amount_to_give = amount_to_give.to_le_bytes();
    escrow_state.bump = bump[0];

    pinocchio_associated_token_account::instructions::Create {
        funding_account: maker,
        account: escrow_ata,
        wallet: escrow_account,
        mint: mint_a,
        token_program: token_program,
        system_program: system_program,
    }
    .invoke()?;

    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: escrow_ata,
        authority: maker,
        amount: amount_to_give,
    }
    .invoke()?;

    Ok(())
}
