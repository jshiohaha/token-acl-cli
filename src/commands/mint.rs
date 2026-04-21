use anyhow::Result;
use clap::Args;
use solana_keypair::Signer;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use spl_token_2022_interface::ID as TOKEN_2022_PROGRAM_ID;

use crate::{cli::AppContext, rpc};

#[derive(Debug, Args)]
pub struct MintArgs {
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long)]
    pub owner: Pubkey,
    #[arg(long)]
    pub token_account: Pubkey,
}

pub async fn run(ctx: &AppContext, args: MintArgs) -> Result<()> {
    let thaw_ix = spl_token_2022_interface::instruction::thaw_account(
        &TOKEN_2022_PROGRAM_ID,
        &args.token_account,
        &args.mint,
        &ctx.payer.pubkey(),
        &[],
    )?;

    let transaction = Transaction::new_signed_with_payer(
        &[thaw_ix],
        Some(&ctx.payer.pubkey()),
        &[ctx.payer.as_ref()],
        ctx.rpc_client.get_latest_blockhash().await?,
    );

    if ctx.shared.simulate {
        let simulation_response = ctx.rpc_client.simulate_transaction(&transaction).await?.value;
        println!("simulation={simulation_response:?}");
        return Ok(());
    }

    let signature = ctx
        .rpc_client
        .send_and_confirm_transaction(&transaction)
        .await?;
    println!("owner={}", args.owner);
    println!(
        "transaction={}",
        rpc::explorer_tx_url(&ctx.shared.rpc_url, &signature.to_string())
    );

    Ok(())
}
