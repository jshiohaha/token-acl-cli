use anyhow::Result;
use clap::Args;
use solana_keypair::{Keypair, Signer};
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use spl_token_2022_interface::{
    ID as TOKEN_2022_PROGRAM_ID,
    extension::{ExtensionType, default_account_state, metadata_pointer, pausable, transfer_fee},
    state::AccountState,
};
use token_acl_client::set_mint_tacl_metadata_ix;

use crate::{cli::AppContext, rpc};

#[derive(Debug, Args)]
pub struct CreateMintArgs {
    #[arg(long, default_value = "That's a nice earf")]
    pub name: String,
    #[arg(long, default_value = "EARF")]
    pub symbol: String,
    #[arg(
        long,
        default_value = "https://peu7kmynwd.ufs.sh/f/iIp4bHYDfTSxsaQFTJGGJPwyqm5O68V9pxuQhtbfAZKzUkdX"
    )]
    pub uri: String,
    #[arg(
        long,
        default_value_t = token_acl_gate_client::programs::TOKEN_ACL_GATE_PROGRAM_ID.to_bytes().into()
    )]
    pub gate_program_id: Pubkey,
    #[arg(long, default_value_t = 10)]
    pub transfer_fee_basis_points: u16,
    #[arg(long, default_value_t = 1_000_000_000_000)]
    pub maximum_transfer_fee: u64,
    #[arg(long, default_value_t = 6)]
    pub decimals: u8,
    #[arg(long)]
    pub freeze_authority: Option<Pubkey>,
    #[arg(long)]
    pub pause_authority: Option<Pubkey>,
    #[arg(long)]
    pub permanent_delegate: Option<Pubkey>,
}

fn metadata_with_acl_attribute(
    token_metadata: &spl_token_metadata_interface::state::TokenMetadata,
) -> Result<usize> {
    let mut size = 0;
    size += 2;
    size += 2;
    size += 32;
    size += 32;
    size += token_metadata.tlv_size_of()?;

    Ok(size)
}

async fn initialize_mint_ixs(
    ctx: &AppContext,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    token_metadata: &spl_token_metadata_interface::state::TokenMetadata,
    metadata_pointer_authority: &Pubkey,
    metadata_authority: &Pubkey,
    transfer_fee_config_authority: Option<&Pubkey>,
    withdraw_withheld_authority: Option<&Pubkey>,
    pause_authority: &Pubkey,
    permanent_delegate: &Pubkey,
    transfer_fee_basis_points: u16,
    maximum_transfer_fee: u64,
    decimals: u8,
    mint: &Pubkey,
    gate_program_id: &Pubkey,
) -> Result<Vec<solana_instruction::Instruction>> {
    let fixed_extensions = &[
        ExtensionType::DefaultAccountState,
        ExtensionType::TransferFeeConfig,
        ExtensionType::Pausable,
        ExtensionType::PermanentDelegate,
        ExtensionType::MetadataPointer,
    ];

    let mint_len = ExtensionType::try_calculate_account_len::<spl_token_2022_interface::state::Mint>(
        fixed_extensions,
    )?;
    let metadata_space = metadata_with_acl_attribute(token_metadata)?;
    let mint_rent = ctx
        .rpc_client
        .get_minimum_balance_for_rent_exemption(mint_len + metadata_space)
        .await?;

    Ok(vec![
        solana_system_interface::instruction::create_account(
            &ctx.payer.pubkey(),
            mint,
            mint_rent,
            mint_len as u64,
            &TOKEN_2022_PROGRAM_ID,
        ),
        default_account_state::instruction::initialize_default_account_state(
            &TOKEN_2022_PROGRAM_ID,
            mint,
            &AccountState::Frozen,
        )?,
        transfer_fee::instruction::initialize_transfer_fee_config(
            &TOKEN_2022_PROGRAM_ID,
            mint,
            transfer_fee_config_authority,
            withdraw_withheld_authority,
            transfer_fee_basis_points,
            maximum_transfer_fee,
        )?,
        pausable::instruction::initialize(&TOKEN_2022_PROGRAM_ID, mint, pause_authority)?,
        spl_token_2022_interface::instruction::initialize_permanent_delegate(
            &TOKEN_2022_PROGRAM_ID,
            mint,
            permanent_delegate,
        )?,
        metadata_pointer::instruction::initialize(
            &TOKEN_2022_PROGRAM_ID,
            mint,
            Some(*metadata_pointer_authority),
            Some(*mint),
        )?,
        spl_token_2022_interface::instruction::initialize_mint(
            &TOKEN_2022_PROGRAM_ID,
            mint,
            mint_authority,
            freeze_authority,
            decimals,
        )?,
        spl_token_metadata_interface::instruction::initialize(
            &TOKEN_2022_PROGRAM_ID,
            mint,
            metadata_authority,
            mint,
            mint_authority,
            token_metadata.name.clone(),
            token_metadata.symbol.clone(),
            token_metadata.uri.clone(),
        ),
        set_mint_tacl_metadata_ix(mint, metadata_authority, gate_program_id),
    ])
}

pub async fn run(ctx: &AppContext, args: CreateMintArgs) -> Result<()> {
    rpc::airdrop_if_localnet(
        &ctx.rpc_client,
        &ctx.shared.rpc_url,
        &ctx.payer.pubkey(),
        1_000_000_000,
    )
    .await?;

    let balance = ctx.rpc_client.get_balance(&ctx.payer.pubkey()).await?;
    println!("address={}, balance={balance}", ctx.payer.pubkey());

    let authority = ctx.payer.clone();
    let mint_kp = Keypair::new();
    let freeze_authority = args.freeze_authority.unwrap_or_else(|| authority.pubkey());
    let pause_authority = args.pause_authority.unwrap_or_else(|| authority.pubkey());
    let permanent_delegate = args
        .permanent_delegate
        .unwrap_or_else(|| authority.pubkey());

    println!("mint={}", mint_kp.pubkey());

    let token_metadata = spl_token_metadata_interface::state::TokenMetadata {
        name: args.name,
        symbol: args.symbol,
        uri: args.uri,
        additional_metadata: vec![("token_acl".to_string(), args.gate_program_id.to_string())],
        ..Default::default()
    };

    let mint_ixs = initialize_mint_ixs(
        ctx,
        &authority.pubkey(),
        Some(&freeze_authority),
        &token_metadata,
        &authority.pubkey(),
        &authority.pubkey(),
        Some(&authority.pubkey()),
        Some(&authority.pubkey()),
        &pause_authority,
        &permanent_delegate,
        args.transfer_fee_basis_points,
        args.maximum_transfer_fee,
        args.decimals,
        &mint_kp.pubkey(),
        &args.gate_program_id,
    )
    .await?;

    let mint_tx = Transaction::new_signed_with_payer(
        &mint_ixs,
        Some(&ctx.payer.pubkey()),
        &[ctx.payer.as_ref(), &mint_kp, authority.as_ref()],
        ctx.rpc_client.get_latest_blockhash().await?,
    );

    if ctx.shared.simulate {
        let simulation_response = ctx.rpc_client.simulate_transaction(&mint_tx).await?.value;
        println!("simulation={simulation_response:?}");
        return Ok(());
    }

    let signature = ctx
        .rpc_client
        .send_and_confirm_transaction(&mint_tx)
        .await?;
    println!(
        "transaction={}",
        rpc::explorer_tx_url(&ctx.shared.rpc_url, &signature.to_string())
    );

    Ok(())
}
