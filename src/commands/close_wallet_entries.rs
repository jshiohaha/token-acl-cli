use anyhow::{Result, anyhow, bail};
use clap::Args;
use solana_account_decoder_client_types::UiAccountEncoding;
use solana_keypair::Signer;
use solana_pubkey::Pubkey;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, RpcFilterType},
};
use solana_transaction::Transaction;
use token_acl_gate_client::{
    accounts::{WALLET_ENTRY_DISCRIMINATOR, WalletEntry},
    instructions::RemoveWalletBuilder,
};

use crate::{cli::AppContext, rpc};

const WALLET_ENTRY_ACCOUNT_SIZE: u64 = 65;

#[derive(Debug, Args)]
pub struct CloseWalletEntriesArgs {
    #[arg(long)]
    pub list_config: Pubkey,
    #[arg(long, default_value_t = 8)]
    pub batch_size: usize,
}

pub async fn run(ctx: &AppContext, args: CloseWalletEntriesArgs) -> Result<()> {
    if args.batch_size == 0 {
        bail!("batch size must be greater than zero");
    }

    #[allow(deprecated)]
    let accounts = ctx
        .rpc_client
        .get_program_accounts_with_config(
            &token_acl_gate_client::programs::TOKEN_ACL_GATE_PROGRAM_ID,
            RpcProgramAccountsConfig {
                filters: Some(vec![
                    RpcFilterType::DataSize(WALLET_ENTRY_ACCOUNT_SIZE),
                    RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                        0,
                        vec![WALLET_ENTRY_DISCRIMINATOR],
                    )),
                    RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                        33,
                        args.list_config.to_bytes().to_vec(),
                    )),
                ]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .await
        .map_err(|err| {
            anyhow!(
                "failed to fetch wallet entries for list {}: {err}",
                args.list_config
            )
        })?;

    let instructions = accounts
        .iter()
        .map(|(wallet_entry_address, account)| {
            let decoded = WalletEntry::from_bytes(&account.data).map_err(|err| {
                anyhow!(
                    "failed to decode wallet entry {}: {err}",
                    wallet_entry_address
                )
            })?;

            if decoded.list_config != args.list_config {
                bail!(
                    "wallet entry {} belongs to {}, expected {}",
                    wallet_entry_address,
                    decoded.list_config,
                    args.list_config
                );
            }

            let mut builder = RemoveWalletBuilder::new();
            builder
                .authority(ctx.payer.pubkey())
                .list_config(args.list_config)
                .wallet_entry(*wallet_entry_address);

            Ok::<_, anyhow::Error>(builder.instruction())
        })
        .collect::<Result<Vec<_>>>()?;

    println!(
        "list_config={}, wallet_entries={}, batch_size={}",
        args.list_config,
        instructions.len(),
        args.batch_size
    );

    if ctx.shared.simulate {
        for (index, chunk) in instructions.chunks(args.batch_size).enumerate() {
            let transaction = Transaction::new_signed_with_payer(
                chunk,
                Some(&ctx.payer.pubkey()),
                &[ctx.payer.as_ref()],
                ctx.rpc_client.get_latest_blockhash().await?,
            );
            let simulation_response = ctx
                .rpc_client
                .simulate_transaction(&transaction)
                .await?
                .value;
            println!("batch={} simulation={simulation_response:?}", index + 1);
        }

        return Ok(());
    }

    for (index, chunk) in instructions.chunks(args.batch_size).enumerate() {
        let transaction = Transaction::new_signed_with_payer(
            chunk,
            Some(&ctx.payer.pubkey()),
            &[ctx.payer.as_ref()],
            ctx.rpc_client.get_latest_blockhash().await?,
        );
        let signature = ctx
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .await?;
        println!(
            "batch={} transaction={}",
            index + 1,
            rpc::explorer_tx_url(&ctx.shared.rpc_url, &signature.to_string())
        );
    }

    Ok(())
}
