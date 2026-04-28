use std::sync::Arc;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use solana_commitment_config::CommitmentConfig;
use solana_keypair::Keypair;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use bincode::serialize;

use crate::{commands, signer};

#[derive(Debug, Parser)]
#[command(name = "gated-mint-cli")]
#[command(about = "Manage gated Token-2022 mints")]
pub struct Cli {
    #[command(flatten)]
    pub shared: SharedArgs,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args, Clone)]
pub struct SharedArgs {
    #[arg(long, default_value = "http://127.0.0.1:8899")]
    pub rpc_url: String,
    #[arg(
        long,
        value_name = "BASE58_OR_BYTES",
        help = "Signer keypair as a base58 secret key string or a 64-byte list like [1,2,...]"
    )]
    pub keypair: String,
    #[arg(long)]
    pub simulate: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    CreateAlt(commands::alt::CreateAltArgs),
    ExtendAlt(commands::alt::ExtendAltArgs),
    CreateConfig(commands::acl::CreateConfigArgs),
    DeleteConfig(commands::acl::DeleteConfigArgs),
    SetAuthority(commands::acl::SetAuthorityArgs),
    SetGatingProgram(commands::acl::SetGatingProgramArgs),
    SetInstructions(commands::acl::SetInstructionsArgs),
    Freeze(commands::acl::FreezeArgs),
    FreezePermissionless(commands::acl::PermissionlessTokenAccountArgs),
    Thaw(commands::acl::ThawArgs),
    ThawPermissionless(commands::acl::PermissionlessTokenAccountArgs),
    CreateAtaAndThawPermissionless(commands::acl::CreateAtaAndThawPermissionlessArgs),
    CloseWalletEntries(commands::close_wallet_entries::CloseWalletEntriesArgs),
    CloseWalletEntry(commands::close_wallet_entry::CloseWalletEntryArgs),
    CreateWalletEntry(commands::create_wallet_entry::CreateWalletEntryArgs),
    CreateList(commands::create_list::CreateListArgs),
    CreateMint(commands::create_mint::CreateMintArgs),
    DeleteList(commands::delete_list::DeleteListArgs),
    Mint(commands::mint::MintArgs),
}

pub struct AppContext {
    pub rpc_client: Arc<RpcClient>,
    pub payer: Arc<Keypair>,
    pub shared: SharedArgs,
}

impl AppContext {
    fn new(shared: SharedArgs) -> Result<Self> {
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            shared.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        ));
        let payer = Arc::new(signer::parse_keypair_arg(&shared.keypair)?);

        Ok(Self {
            rpc_client,
            payer,
            shared,
        })
    }
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let ctx = AppContext::new(cli.shared)?;

    match cli.command {
        Command::CreateAlt(args) => commands::alt::run_create(&ctx, args).await,
        Command::ExtendAlt(args) => commands::alt::run_extend(&ctx, args).await,
        Command::CreateConfig(args) => commands::acl::run_create_config(&ctx, args).await,
        Command::DeleteConfig(args) => commands::acl::run_delete_config(&ctx, args).await,
        Command::SetAuthority(args) => commands::acl::run_set_authority(&ctx, args).await,
        Command::SetGatingProgram(args) => commands::acl::run_set_gating_program(&ctx, args).await,
        Command::SetInstructions(args) => commands::acl::run_set_instructions(&ctx, args).await,
        Command::Freeze(args) => commands::acl::run_freeze(&ctx, args).await,
        Command::FreezePermissionless(args) => {
            commands::acl::run_freeze_permissionless(&ctx, args).await
        }
        Command::Thaw(args) => commands::acl::run_thaw(&ctx, args).await,
        Command::ThawPermissionless(args) => {
            commands::acl::run_thaw_permissionless(&ctx, args).await
        }
        Command::CreateAtaAndThawPermissionless(args) => {
            commands::acl::run_create_ata_and_thaw_permissionless(&ctx, args).await
        }
        Command::CloseWalletEntries(args) => commands::close_wallet_entries::run(&ctx, args).await,
        Command::CloseWalletEntry(args) => commands::close_wallet_entry::run(&ctx, args).await,
        Command::CreateWalletEntry(args) => commands::create_wallet_entry::run(&ctx, args).await,
        Command::CreateList(args) => commands::create_list::run(&ctx, args).await,
        Command::CreateMint(args) => commands::create_mint::run(&ctx, args).await,
        Command::DeleteList(args) => commands::delete_list::run(&ctx, args).await,
        Command::Mint(args) => commands::mint::run(&ctx, args).await,
    }
}
