use anyhow::{Result, anyhow};
use clap::Args;
use solana_keypair::Signer;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use spl_token_2022_interface::{
    ID as TOKEN_2022_PROGRAM_ID,
    extension::PodStateWithExtensions,
    pod::PodMint,
};

use crate::{cli::AppContext, rpc};

#[derive(Debug, Args)]
pub struct MintArgs {
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long)]
    pub owner: Pubkey,
    #[arg(long)]
    pub token_account: Pubkey,
    /// UI amount (e.g. 1.5); decimals are fetched from the mint and applied automatically
    #[arg(long)]
    pub ui_amount: f64,
}

pub async fn run(ctx: &AppContext, args: MintArgs) -> Result<()> {
    let mint_data = ctx
        .rpc_client
        .get_account_data(&args.mint)
        .await
        .map_err(|err| anyhow!("failed to fetch mint {}: {err}", args.mint))?;
    let decimals = PodStateWithExtensions::<PodMint>::unpack(&mint_data)
        .map_err(|err| anyhow!("failed to unpack mint {}: {err}", args.mint))?
        .base
        .decimals;
    let raw_amount = (args.ui_amount * 10f64.powi(decimals as i32)).round() as u64;

    let mint_to_ix = spl_token_2022_interface::instruction::mint_to(
        &TOKEN_2022_PROGRAM_ID,
        &args.mint,
        &args.token_account,
        &ctx.payer.pubkey(),
        &[],
        raw_amount,
    )?;

    let transaction = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&ctx.payer.pubkey()),
        &[ctx.payer.as_ref()],
        ctx.rpc_client.get_latest_blockhash().await?,
    );

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
    println!("mint={}", args.mint);
    println!("owner={}", args.owner);
    println!("token_account={}", args.token_account);
    println!("amount={}", args.ui_amount);
    println!(
        "transaction={}",
        rpc::explorer_tx_url(&ctx.shared.rpc_url, &signature.to_string())
    );

    Ok(())
}
