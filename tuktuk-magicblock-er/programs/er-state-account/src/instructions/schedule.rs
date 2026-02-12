use std::str::FromStr;

use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{prelude::*, InstructionData};
use ephemeral_vrf_sdk::anchor::vrf;
use tuktuk_program::{
    compile_transaction,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
        types::TriggerV0,
    },
    types::QueueTaskArgsV0,
    TransactionSourceV0,
};

use crate::state::UserAccount;

#[vrf]
#[derive(Accounts)]
pub struct Schedule<'info> {
    #[account(
        mut,
        address = Pubkey::from_str("AJvghTd4HpB7vQzJJX42HhCak5cyQAkh3u8SGN1JQVVG").unwrap()
    )]
    pub user: Signer<'info>,
    /// CHECK: This is safe because we don't read or write from this account
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: queue
    #[account(mut, address = ephemeral_vrf_sdk::consts::DEFAULT_QUEUE)]
    pub oracle_queue: AccountInfo<'info>,

    // TUKTUK
    #[account(mut)]
    /// CHECK: Don't need to parse this account, just using it in CPI
    pub task_queue: UncheckedAccount<'info>,
    /// CHECK: Don't need to parse this account, just using it in CPI
    pub task_queue_authority: UncheckedAccount<'info>,
    /// CHECK: Initialized in CPI
    #[account(mut)]
    pub task: UncheckedAccount<'info>,
    /// CHECK: Via seeds
    #[account(
        mut,
        seeds = [b"queue_authority"],
        bump
    )]
    pub queue_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub tuktuk_program: Program<'info, Tuktuk>,
}

impl<'info> Schedule<'info> {
    pub fn schedule(&mut self, task_id: u16, bumps: &ScheduleBumps) -> Result<()> {
        let (compiled_tx, _) = compile_transaction(
            vec![Instruction {
                program_id: crate::ID,
                accounts: crate::__client_accounts_request_data::RequestData {
                    payer: self.user.key(),
                    user: self.user_account.key(),
                    oracle_queue: self.oracle_queue.key(),
                    program_identity: self.program_identity.key(),
                    vrf_program: self.vrf_program.key(),
                    slot_hashes: self.slot_hashes.key(),
                    system_program: self.system_program.key(),
                }
                .to_account_metas(None)
                .to_vec(),
                data: crate::instruction::RequestData {
                    client_seed: task_id as u8,
                }
                .data(),
            }],
            vec![],
        )
        .unwrap();

        queue_task_v0(
            CpiContext::new_with_signer(
                self.tuktuk_program.to_account_info(),
                QueueTaskV0 {
                    payer: self.user.to_account_info(),
                    queue_authority: self.queue_authority.to_account_info(),
                    task_queue: self.task_queue.to_account_info(),
                    task_queue_authority: self.task_queue_authority.to_account_info(),
                    task: self.task.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
                &[&["queue_authority".as_bytes(), &[bumps.queue_authority]]],
            ),
            QueueTaskArgsV0 {
                trigger: TriggerV0::Now,
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: Some(1000001),
                free_tasks: 1,
                id: task_id,
                description: "test".to_string(),
            },
        )?;

        Ok(())
    }
}
