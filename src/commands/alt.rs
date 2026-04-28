use anyhow::Result;
use clap::Args;
use solana_address_lookup_table_interface::instruction::{
    create_lookup_table, extend_lookup_table,
};
use solana_keypair::Signer;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

use crate::{cli::AppContext, rpc};

#[derive(Debug, Args)]
pub struct CreateAltArgs {}

pub async fn run_create(ctx: &AppContext, _args: CreateAltArgs) -> Result<()> {
    let authority = ctx.payer.pubkey();

    let recent_slot = ctx.rpc_client.get_slot().await?;

    let (create_ix, lookup_table_address) =
        create_lookup_table(authority, authority, recent_slot);

    let transaction = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&authority),
        &[ctx.payer.as_ref()],
        ctx.rpc_client.get_latest_blockhash().await?,
    );

    println!("authority={authority}");
    println!("recent_slot={recent_slot}");
    println!("lookup_table={lookup_table_address}");

    if ctx.shared.simulate {
        let result = ctx
            .rpc_client
            .simulate_transaction(&transaction)
            .await?
            .value;
        println!("simulation={result:?}");
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

#[derive(Debug, Args)]
pub struct ExtendAltArgs {
    /// Address of the lookup table to extend
    #[arg(long)]
    pub lookup_table: Pubkey,
    /// Addresses to add (repeat flag for multiple: --address <ADDR1> --address <ADDR2>)
    #[arg(long = "address", required = true)]
    pub addresses: Vec<Pubkey>,
}

pub async fn run_extend(ctx: &AppContext, args: ExtendAltArgs) -> Result<()> {
    let authority = ctx.payer.pubkey();

    let extend_ix = extend_lookup_table(
        args.lookup_table,
        authority,
        Some(authority),
        args.addresses.clone(),
    );

    let transaction = Transaction::new_signed_with_payer(
        &[extend_ix],
        Some(&authority),
        &[ctx.payer.as_ref()],
        ctx.rpc_client.get_latest_blockhash().await?,
    );

    println!("lookup_table={}", args.lookup_table);
    println!("authority={authority}");
    println!("addresses_added={}", args.addresses.len());
    for addr in &args.addresses {
        println!("  {addr}");
    }

    if ctx.shared.simulate {
        let result = ctx
            .rpc_client
            .simulate_transaction(&transaction)
            .await?
            .value;
        println!("simulation={result:?}");
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
