use anchor_lang::prelude::*;
use mpl_core::{
    instructions::{
        AddCollectionExternalPluginAdapterV1CpiBuilder, AddCollectionPluginV1CpiBuilder,
        CreateCollectionV2CpiBuilder,
    },
    types::{
        Attribute, Attributes, ExternalCheckResult, ExternalPluginAdapterInitInfo,
        HookableLifecycleEvent, OracleInitInfo, Plugin, PluginAuthority,
    },
    ID as MPL_CORE_ID,
};

use crate::helpers::ORACLE_ACCOUNT;

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub collection: Signer<'info>,
    /// CHECK: PDA Update authority
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> CreateCollection<'info> {
    pub fn create_collection(
        &mut self,
        name: String,
        uri: String,
        bumps: &CreateCollectionBumps,
    ) -> Result<()> {
        // Signer seeds for the update authority
        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        // Create the collection with CPI builder
        CreateCollectionV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.payer.to_account_info())
            .update_authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .invoke_signed(&[signer_seeds])?;

        // add total_staked attribute
        AddCollectionPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.payer.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes {
                attribute_list: vec![Attribute {
                    key: "total_staked".to_string(),
                    value: "0".to_string(),
                }],
            }))
            .init_authority(PluginAuthority::UpdateAuthority)
            .invoke_signed(&[signer_seeds])?;

        AddCollectionExternalPluginAdapterV1CpiBuilder::new(
            &self.mpl_core_program.to_account_info(),
        )
        .collection(&self.collection.to_account_info())
        .payer(&self.payer.to_account_info())
        .authority(Some(&self.update_authority.to_account_info()))
        .system_program(&self.system_program.to_account_info())
        .init_info(ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
            base_address: ORACLE_ACCOUNT,
            init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Transfer,
                // can reject the lifecycle event
                // https://github.com/metaplex-foundation/mpl-core/blob/e021ca45e55285bb1f95789f010ac62e64caad1b/programs/mpl-core/src/plugins/lifecycle.rs#L53
                ExternalCheckResult { flags: 4 },
            )],
            base_address_config: None,
            results_offset: Some(mpl_core::types::ValidationResultsOffset::Anchor),
        }))
        .invoke_signed(&[signer_seeds])?;

        Ok(())
    }
}
