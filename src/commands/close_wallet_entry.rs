use anyhow::Result;
use clap::Args;
use solana_keypair::Signer;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use token_acl_gate_client::{accounts::WalletEntry, instructions::RemoveWalletBuilder};

use crate::{cli::AppContext, rpc};

#[derive(Debug, Args)]
pub struct CloseWalletEntryArgs {
    #[arg(long)]
    pub list_config: Pubkey,
    #[arg(long)]
    pub wallet: Pubkey,
}

pub async fn run(ctx: &AppContext, args: CloseWalletEntryArgs) -> Result<()> {
    let authority = ctx.payer.pubkey();
    let wallet_entry = WalletEntry::find_pda(&args.list_config, &args.wallet).0;

    let mut builder = RemoveWalletBuilder::new();
    builder
        .authority(authority)
        .list_config(args.list_config)
        .wallet_entry(wallet_entry);

    let transaction = Transaction::new_signed_with_payer(
        &[builder.instruction()],
        Some(&authority),
        &[ctx.payer.as_ref()],
        ctx.rpc_client.get_latest_blockhash().await?,
    );

    println!("list_config={}", args.list_config);
    println!("wallet={}", args.wallet);
    println!("wallet_entry={wallet_entry}");

    if ctx.shared.simulate {
        let simulation_response = ctx
            .rpc_client
            .simulate_transaction(&transaction)
            .await?
            .value;
        println!("simulation={simulation_response:?}");
        return Ok(());
    }

    let signature = ctx
        .rpc_client
        .send_and_confirm_transaction(&transaction)
        .await?;
    println!(
        "transaction={}",
        rpc::explorer_tx_url(&ctx.shared.rpc_url, &signature.to_string())
    );

    Ok(())
}
