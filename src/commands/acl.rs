use anyhow::{Result, anyhow, bail};
use clap::{ArgGroup, Args};
use solana_commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_keypair::{Keypair, Signer};
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_transaction::Transaction;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022_interface::{
    ID as TOKEN_2022_PROGRAM_ID,
    extension::{BaseStateWithExtensions, PodStateWithExtensions, StateWithExtensions},
    pod::PodMint,
    state::{Account, AccountState},
};
use spl_token_metadata_interface::state::TokenMetadata;
use token_acl_client::set_mint_tacl_metadata_ix;

use crate::{cli::AppContext, rpc, signer};

#[derive(Debug, Args)]
pub struct CreateConfigArgs {
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long)]
    pub gating_program: Option<Pubkey>,
    #[arg(
        long,
        value_name = "BASE58_OR_BYTES",
        help = "Optional freeze-authority signer as a base58 secret key string or 64-byte list"
    )]
    pub freeze_authority: Option<String>,
}

#[derive(Debug, Args)]
pub struct DeleteConfigArgs {
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long)]
    pub receiver: Option<Pubkey>,
}

#[derive(Debug, Args)]
pub struct SetAuthorityArgs {
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long)]
    pub new_authority: Pubkey,
}

#[derive(Debug, Args)]
pub struct SetGatingProgramArgs {
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long)]
    pub new_gating_program: Pubkey,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("thaw_setting")
        .args(["enable_thaw", "disable_thaw"])
        .required(true)
))]
#[command(group(
    ArgGroup::new("freeze_setting")
        .args(["enable_freeze", "disable_freeze"])
        .required(true)
))]
pub struct SetInstructionsArgs {
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long)]
    pub enable_thaw: bool,
    #[arg(long)]
    pub disable_thaw: bool,
    #[arg(long)]
    pub enable_freeze: bool,
    #[arg(long)]
    pub disable_freeze: bool,
}

#[derive(Debug, Args)]
pub struct FreezeArgs {
    #[arg(long)]
    pub token_account: Pubkey,
}

#[derive(Debug, Args)]
pub struct ThawArgs {
    #[arg(long)]
    pub token_account: Pubkey,
}

#[derive(Debug, Args)]
pub struct PermissionlessTokenAccountArgs {
    #[arg(long)]
    pub mint: Option<Pubkey>,
    #[arg(long)]
    pub token_account: Option<Pubkey>,
    #[arg(long)]
    pub owner: Option<Pubkey>,
}

#[derive(Debug, Args)]
pub struct CreateAtaAndThawPermissionlessArgs {
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long)]
    pub owner: Pubkey,
}

pub async fn run_create_config(ctx: &AppContext, args: CreateConfigArgs) -> Result<()> {
    let freeze_authority = args
        .freeze_authority
        .as_deref()
        .map(signer::parse_keypair_arg)
        .transpose()?;

    let config = token_acl_client::accounts::MintConfig::find_pda(&args.mint).0;
    let authority = freeze_authority
        .as_ref()
        .map(Signer::pubkey)
        .unwrap_or_else(|| ctx.payer.pubkey());

    println!("payer: {:?}", ctx.payer.pubkey());
    println!("mint: {:?}", args.mint);
    println!("authority: {:?}", authority);

    let create_config_ix = token_acl_client::instructions::CreateConfigBuilder::new()
        .authority(authority)
        .payer(ctx.payer.pubkey())
        .mint(args.mint)
        .mint_config(config)
        .gating_program(args.gating_program.unwrap_or_default())
        .instruction();

    let mut signers: Vec<&Keypair> = vec![ctx.payer.as_ref()];
    if let Some(freeze_authority) = freeze_authority.as_ref() {
        signers.push(freeze_authority);
    }

    let transaction = signed_transaction(ctx, &[create_config_ix], &signers).await?;
    execute_transaction(ctx, &transaction, false).await?;
    println!("mint_config={config}");

    Ok(())
}

pub async fn run_delete_config(ctx: &AppContext, args: DeleteConfigArgs) -> Result<()> {
    let receiver = args.receiver.unwrap_or_else(|| ctx.payer.pubkey());
    let config = token_acl_client::accounts::MintConfig::find_pda(&args.mint).0;

    let ix = token_acl_client::instructions::DeleteConfigBuilder::new()
        .authority(ctx.payer.pubkey())
        .receiver(receiver)
        .mint(args.mint)
        .mint_config(config)
        .instruction();

    let transaction = signed_transaction(ctx, &[ix], &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, false).await?;

    Ok(())
}

pub async fn run_set_authority(ctx: &AppContext, args: SetAuthorityArgs) -> Result<()> {
    let config = token_acl_client::accounts::MintConfig::find_pda(&args.mint).0;
    let ix = token_acl_client::instructions::SetAuthorityBuilder::new()
        .authority(ctx.payer.pubkey())
        .new_authority(args.new_authority)
        .mint_config(config)
        .instruction();

    let transaction = signed_transaction(ctx, &[ix], &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, false).await?;

    Ok(())
}

pub async fn run_set_gating_program(ctx: &AppContext, args: SetGatingProgramArgs) -> Result<()> {
    let config = token_acl_client::accounts::MintConfig::find_pda(&args.mint).0;
    let update_gating_program_ix = token_acl_client::instructions::SetGatingProgramBuilder::new()
        .authority(ctx.payer.pubkey())
        .new_gating_program(args.new_gating_program)
        .mint_config(config)
        .instruction();

    let mut instructions = vec![update_gating_program_ix];
    maybe_push_metadata_resize_and_set_ixs(
        ctx,
        &args.mint,
        &ctx.payer.pubkey(),
        args.new_gating_program,
        &mut instructions,
    )
    .await?;

    let transaction = signed_transaction(ctx, &instructions, &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, false).await?;

    Ok(())
}

pub async fn run_set_instructions(ctx: &AppContext, args: SetInstructionsArgs) -> Result<()> {
    let config = token_acl_client::accounts::MintConfig::find_pda(&args.mint).0;
    let ix = token_acl_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(ctx.payer.pubkey())
        .thaw_enabled(args.enable_thaw)
        .freeze_enabled(args.enable_freeze)
        .mint_config(config)
        .instruction();

    let transaction = signed_transaction(ctx, &[ix], &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, false).await?;

    Ok(())
}

pub async fn run_freeze(ctx: &AppContext, args: FreezeArgs) -> Result<()> {
    let (mint, _) = fetch_token_account_base(ctx, args.token_account).await?;
    let config = token_acl_client::accounts::MintConfig::find_pda(&mint).0;
    let ix = token_acl_client::instructions::FreezeBuilder::new()
        .authority(ctx.payer.pubkey())
        .mint(mint)
        .token_account(args.token_account)
        .mint_config(config)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .instruction();

    let transaction = signed_transaction(ctx, &[ix], &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, false).await?;

    Ok(())
}

pub async fn run_thaw(ctx: &AppContext, args: ThawArgs) -> Result<()> {
    let (mint, _) = fetch_token_account_base(ctx, args.token_account).await?;
    let config = token_acl_client::accounts::MintConfig::find_pda(&mint).0;
    let ix = token_acl_client::instructions::ThawBuilder::new()
        .authority(ctx.payer.pubkey())
        .mint(mint)
        .token_account(args.token_account)
        .mint_config(config)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .instruction();

    let transaction = signed_transaction(ctx, &[ix], &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, false).await?;

    Ok(())
}

pub async fn run_freeze_permissionless(
    ctx: &AppContext,
    args: PermissionlessTokenAccountArgs,
) -> Result<()> {
    let (mint, token_account, owner, instructions, ata_data, new_ata) =
        resolve_permissionless_accounts(ctx, args).await?;
    let config = token_acl_client::accounts::MintConfig::find_pda(&mint).0;

    let ix = token_acl_client::create_freeze_permissionless_instruction_with_extra_metas(
        &ctx.payer.pubkey(),
        &token_account,
        &mint,
        &config,
        &TOKEN_2022_PROGRAM_ID,
        &owner,
        false,
        |pubkey| {
            let rpc_client = ctx.rpc_client.clone();
            let ata_data = ata_data.clone();
            async move {
                if new_ata && pubkey == token_account {
                    return Ok(Some(ata_data));
                }
                Ok(rpc_client
                    .get_account(&pubkey)
                    .await
                    .ok()
                    .map(|account| account.data))
            }
        },
    )
    .await
    .map_err(|err| anyhow!("failed to build freeze-permissionless instruction: {err}"))?;

    let mut instructions = instructions;
    instructions.push(ix);

    println!("mint={mint}");
    println!("token_account={token_account}");
    println!("owner={owner}");

    let transaction = signed_transaction(ctx, &instructions, &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, true).await?;

    Ok(())
}

pub async fn run_thaw_permissionless(
    ctx: &AppContext,
    args: PermissionlessTokenAccountArgs,
) -> Result<()> {
    let (mint, token_account, owner, instructions, ata_data, new_ata) =
        resolve_permissionless_accounts(ctx, args).await?;
    let config = token_acl_client::accounts::MintConfig::find_pda(&mint).0;

    let ix = token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
        &ctx.payer.pubkey(),
        &token_account,
        &mint,
        &config,
        &TOKEN_2022_PROGRAM_ID,
        &owner,
        false,
        |pubkey| {
            let rpc_client = ctx.rpc_client.clone();
            let ata_data = ata_data.clone();
            async move {
                if new_ata && pubkey == token_account {
                    return Ok(Some(ata_data));
                }
                Ok(rpc_client
                    .get_account(&pubkey)
                    .await
                    .ok()
                    .map(|account| account.data))
            }
        },
    )
    .await
    .map_err(|err| anyhow!("failed to build thaw-permissionless instruction: {err}"))?;

    let mut instructions = instructions;
    instructions.push(ix);

    println!("mint={mint}");
    println!("token_account={token_account}");
    println!("owner={owner}");

    let transaction = signed_transaction(ctx, &instructions, &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, true).await?;

    Ok(())
}

pub async fn run_create_ata_and_thaw_permissionless(
    ctx: &AppContext,
    args: CreateAtaAndThawPermissionlessArgs,
) -> Result<()> {
    let instructions = token_acl_client::create_ata_and_thaw_permissionless(
        &ctx.rpc_client,
        &ctx.payer.pubkey(),
        &args.mint,
        &args.owner,
        false,
    )
    .await
    .map_err(|err| {
        anyhow!("failed to build create-ata-and-thaw-permissionless instructions: {err}")
    })?;

    let token_account = get_associated_token_address_with_program_id(
        &args.owner,
        &args.mint,
        &TOKEN_2022_PROGRAM_ID,
    );

    println!("mint={}", args.mint);
    println!("token_account={token_account}");
    println!("owner={}", args.owner);

    let transaction = signed_transaction(ctx, &instructions, &[ctx.payer.as_ref()]).await?;
    execute_transaction(ctx, &transaction, true).await?;

    Ok(())
}

async fn maybe_push_metadata_resize_and_set_ixs(
    ctx: &AppContext,
    mint: &Pubkey,
    metadata_authority: &Pubkey,
    gating_program: Pubkey,
    instructions: &mut Vec<solana_instruction::Instruction>,
) -> Result<()> {
    let mint_data = ctx
        .rpc_client
        .get_account_data(mint)
        .await
        .map_err(|err| anyhow!("failed to fetch mint data for {mint}: {err}"))?;
    let mint_unpacked = PodStateWithExtensions::<PodMint>::unpack(&mint_data)
        .map_err(|err| anyhow!("failed to unpack mint data for {mint}: {err}"))?;
    let mut metadata = mint_unpacked
        .get_variable_len_extension::<TokenMetadata>()
        .map_err(|err| anyhow!("failed to read metadata for {mint}: {err}"))?;

    let initial_tlv_size = metadata.tlv_size_of()?;
    metadata.set_key_value(
        token_acl_client::TOKEN_ACL_METADATA_KEY.to_string(),
        gating_program.to_string(),
    );
    let new_tlv_size = metadata.tlv_size_of()?;

    if new_tlv_size > initial_tlv_size {
        let rent = ctx
            .rpc_client
            .get_minimum_balance_for_rent_exemption(new_tlv_size - initial_tlv_size)
            .await?;
        instructions.push(solana_system_interface::instruction::transfer(
            &ctx.payer.pubkey(),
            mint,
            rent,
        ));
    }

    instructions.push(set_mint_tacl_metadata_ix(
        mint,
        metadata_authority,
        &gating_program,
    ));

    Ok(())
}

async fn fetch_token_account_base(
    ctx: &AppContext,
    token_account: Pubkey,
) -> Result<(Pubkey, Pubkey)> {
    let account = ctx
        .rpc_client
        .get_account(&token_account)
        .await
        .map_err(|err| anyhow!("failed to fetch token account {token_account}: {err}"))?;

    let unpacked = StateWithExtensions::<Account>::unpack(account.data.as_ref())
        .map_err(|err| anyhow!("failed to decode token account {token_account}: {err}"))?;

    Ok((unpacked.base.mint, unpacked.base.owner))
}

type PermissionlessResolution = (
    Pubkey,
    Pubkey,
    Pubkey,
    Vec<solana_instruction::Instruction>,
    Vec<u8>,
    bool,
);

async fn resolve_permissionless_accounts(
    ctx: &AppContext,
    args: PermissionlessTokenAccountArgs,
) -> Result<PermissionlessResolution> {
    match (args.mint, args.token_account, args.owner) {
        (None, Some(token_account), None) => {
            let (mint, owner) = fetch_token_account_base(ctx, token_account).await?;
            Ok((mint, token_account, owner, Vec::new(), Vec::new(), false))
        }
        (Some(mint), None, Some(owner)) => {
            let token_account =
                get_associated_token_address_with_program_id(&owner, &mint, &TOKEN_2022_PROGRAM_ID);
            let create_ata_ix = create_associated_token_account(
                &ctx.payer.pubkey(),
                &owner,
                &mint,
                &TOKEN_2022_PROGRAM_ID,
            );

            let account = Account {
                mint,
                owner,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Frozen,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            };

            let mut data = vec![0u8; Account::LEN];
            Account::pack(account, &mut data)?;

            Ok((mint, token_account, owner, vec![create_ata_ix], data, true))
        }
        _ => bail!("provide either `--token-account` or the pair `--mint` and `--owner`"),
    }
}

async fn signed_transaction(
    ctx: &AppContext,
    instructions: &[solana_instruction::Instruction],
    signers: &[&Keypair],
) -> Result<Transaction> {
    let blockhash = ctx.rpc_client.get_latest_blockhash().await?;
    Ok(Transaction::new_signed_with_payer(
        instructions,
        Some(&ctx.payer.pubkey()),
        signers,
        blockhash,
    ))
}

async fn execute_transaction(
    ctx: &AppContext,
    transaction: &Transaction,
    skip_preflight: bool,
) -> Result<()> {
    if ctx.shared.simulate {
        let simulation_response = ctx
            .rpc_client
            .simulate_transaction(transaction)
            .await?
            .value;
        println!("simulation={simulation_response:?}");
        return Ok(());
    }

    let signature = if skip_preflight {
        ctx.rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(
                transaction,
                CommitmentConfig {
                    commitment: CommitmentLevel::Confirmed,
                },
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    ..Default::default()
                },
            )
            .await?
    } else {
        ctx.rpc_client
            .send_and_confirm_transaction(transaction)
            .await?
    };

    println!(
        "transaction={}",
        rpc::explorer_tx_url(&ctx.shared.rpc_url, &signature.to_string())
    );

    Ok(())
}
