use anyhow::{Result, anyhow, bail};
use clap::Args;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Signer;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use token_acl_gate_client::{
    accounts::ListConfig,
    instructions::DeleteListBuilder,
};

use crate::{cli::AppContext, rpc};

#[derive(Debug, Args)]
pub struct DeleteListArgs {
    #[arg(long)]
    pub list_config: Pubkey,
}

pub async fn run(ctx: &AppContext, args: DeleteListArgs) -> Result<()> {
    let account = ctx
        .rpc_client
        .get_account(&args.list_config)
        .await
        .map_err(|err| anyhow!("failed to fetch list config {}: {err}", args.list_config))?;

    let decoded = ListConfig::from_bytes(&account.data)
        .map_err(|err| anyhow!("failed to decode list config {}: {err}", args.list_config))?;

    let expected_authority = ctx.payer.pubkey();
    let actual_authority = into_app_pubkey(decoded.authority);
    if actual_authority != expected_authority {
        bail!(
            "list {} authority is {}, signer is {}",
            args.list_config,
            actual_authority,
            expected_authority
        );
    }

    println!(
        "list_config={}, seed={}, mode={}, wallets_count={}",
        args.list_config,
        into_app_pubkey(decoded.seed),
        decoded.mode,
        decoded.wallets_count
    );

    let mut builder = DeleteListBuilder::new();
    builder
        .authority(into_gate_pubkey(expected_authority))
        .list_config(into_gate_pubkey(args.list_config));
    let instruction = into_app_instruction(builder.instruction());

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&expected_authority),
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
    println!(
        "transaction={}",
        rpc::explorer_tx_url(&ctx.shared.rpc_url, &signature.to_string())
    );

    Ok(())
}

fn into_gate_pubkey(pubkey: Pubkey) -> solana_pubkey_v2::Pubkey {
    pubkey.to_bytes().into()
}

fn into_app_pubkey(pubkey: solana_pubkey_v2::Pubkey) -> Pubkey {
    pubkey.to_bytes().into()
}

fn into_app_instruction(ix: solana_instruction_v2::Instruction) -> Instruction {
    Instruction {
        program_id: into_app_pubkey(ix.program_id),
        accounts: ix
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: into_app_pubkey(meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: ix.data,
    }
}
