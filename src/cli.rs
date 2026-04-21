use std::sync::Arc;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use solana_commitment_config::CommitmentConfig;
use solana_keypair::Keypair;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

use crate::{commands, signer};

const DEFAULT_RPC_URL: &str = "https://orca.rpcpool.com/ae9a156f6bdd344c8267465eb432";

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
    #[arg(long, default_value = DEFAULT_RPC_URL)]
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
    CloseWalletEntries(commands::close_wallet_entries::CloseWalletEntriesArgs),
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
        Command::CloseWalletEntries(args) => commands::close_wallet_entries::run(&ctx, args).await,
        Command::CreateMint(args) => commands::create_mint::run(&ctx, args).await,
        Command::DeleteList(args) => commands::delete_list::run(&ctx, args).await,
        Command::Mint(args) => commands::mint::run(&ctx, args).await,
    }
}
