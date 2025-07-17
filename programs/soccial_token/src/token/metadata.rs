// ===========================================================================
// Metadata Management Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module handles **on-chain metadata creation and updates** for the
// Soccial Token, using the Metaplex Token Metadata program. It allows the URI
// associated with the token to be created (if not yet set) or updated securely,
// while verifying proper ownership and permissions.
//
// ---------------------------------------------------------------------------
// ## Features:
// - Creates metadata via CPI if it doesn't exist (CreateMetadataAccountV3)
// - Updates existing metadata URI (UpdateAsUpdateAuthorityV2)
// - Uses PDA signer (`mint_authority`) for authorized actions
// - Supports programmatic updates via instruction parsing
//
// ---------------------------------------------------------------------------
// ## Components:
// - `update_metadata_uri()`: Creates or updates the on-chain metadata URI
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use mpl_token_metadata::{
    instructions::{
        CreateMetadataAccountV3CpiBuilder, UpdateAsUpdateAuthorityV2CpiBuilder
    },
    types::{Data, DataV2},
};

use crate::token::ChangeUpdateAuthority;

/// ===========================================================================
/// Function: update_metadata_uri
/// ---------------------------------------------------------------------------
/// Creates or updates the on-chain URI for the Soccial Token metadata using the
/// Metaplex Token Metadata program. This function is responsible for initializing
/// the metadata if it doesn't yet exist, or updating the URI field if it does.
///
/// ## Behavior:
/// - If metadata account is empty → calls `CreateMetadataAccountV3`
/// - Otherwise → calls `UpdateAsUpdateAuthorityV2`
/// - Uses `mint_authority` PDA as signer for CPI authority
/// - Can be invoked from a higher-level dispatcher using instruction arguments
///
/// ## Inputs:
/// - `ctx`: Context containing all required accounts
/// - `new_uri`: A valid URI (e.g., IPFS link) to be stored in metadata
///
/// ## Permissions:
/// - Caller must be the contract owner or authorized update authority
///
/// ## Notes:
/// - The URI will be visible on all Solana explorers and marketplaces
/// - Ensures the metadata remains mutable for future updates
/// ===========================================================================

pub fn update_metadata_uri(
    ctx: Context<ChangeUpdateAuthority>,
    new_uri: String,
) -> Result<()> {
    let bump = ctx.bumps.mint_authority;
    let bump_bytes = [bump];
    let seeds: &[&[u8]] = &[b"mint_authority", &bump_bytes];
    let signer_seeds: &[&[&[u8]]] = &[seeds];

    // Define metadata fields
    let data = Data {
        name: "Soccial Token".to_string(),
        symbol: "SCTK".to_string(),
        uri: new_uri.clone(),
        seller_fee_basis_points: 0,
        creators: None,
    };

    // If metadata account doesn't exist, create it via CPI
    if ctx.accounts.metadata.data_is_empty() {
          CreateMetadataAccountV3CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .mint(&ctx.accounts.mint.to_account_info())
            .mint_authority(&ctx.accounts.mint_authority.to_account_info())
            .payer(&ctx.accounts.caller.to_account_info())
            .update_authority(&ctx.accounts.mint_authority.to_account_info(), true)
            .system_program(&ctx.accounts.system_program.to_account_info())
            .rent(Some(&ctx.accounts.rent.to_account_info()))
            .data(DataV2 {
                name: data.name.clone(),
                symbol: data.symbol.clone(),
                uri: new_uri.clone(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            })
            .is_mutable(true)
        .invoke_signed(signer_seeds)?;
    
    } else {
        // Update metadata URI (whether it was created now or already existed)
        UpdateAsUpdateAuthorityV2CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .authority(&ctx.accounts.mint_authority.to_account_info())
            .mint(&ctx.accounts.mint.to_account_info())
            .metadata(&ctx.accounts.metadata)
            .payer(&ctx.accounts.caller.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
            .data(data)
            .is_mutable(true)
            .invoke_signed(signer_seeds)?;
    }

   
    Ok(())
}
