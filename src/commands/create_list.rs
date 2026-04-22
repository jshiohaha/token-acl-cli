use anyhow::Result;
use clap::{Args, ValueEnum};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::{Keypair, Signer};
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use token_acl_gate_client::{accounts::ListConfig, instructions::CreateListBuilder, types::Mode};

use crate::{cli::AppContext, rpc};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ListModeArg {
    Allow,
    AllowAllEoas,
    Block,
}

impl From<ListModeArg> for Mode {
    fn from(value: ListModeArg) -> Self {
        match value {
            ListModeArg::Allow => Mode::Allow,
            ListModeArg::AllowAllEoas => Mode::AllowAllEoas,
            ListModeArg::Block => Mode::Block,
        }
    }
}

#[derive(Debug, Args)]
pub struct CreateListArgs {
    #[arg(long, value_enum, default_value_t = ListModeArg::Allow)]
    pub mode: ListModeArg,
    #[arg(long)]
    pub seed: Option<Pubkey>,
}

pub async fn run(ctx: &AppContext, args: CreateListArgs) -> Result<()> {
    let authority = into_app_pubkey(ctx.payer.pubkey().to_bytes().into());
    let seed = args
        .seed
        .unwrap_or_else(|| into_app_pubkey(Keypair::new().pubkey().to_bytes().into()));
    let list_config = into_app_pubkey(ListConfig::find_pda(&into_gate_pubkey(authority), &into_gate_pubkey(seed)).0);

    let mut builder = CreateListBuilder::new();
    builder
        .authority(into_gate_pubkey(authority))
        .payer(into_gate_pubkey(authority))
        .seed(into_gate_pubkey(seed))
        .mode(args.mode.into())
        .list_config(into_gate_pubkey(list_config));
    let instruction = into_app_instruction(builder.instruction());

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&authority),
        &[ctx.payer.as_ref()],
        ctx.rpc_client.get_latest_blockhash().await?,
    );

    println!("list_config={list_config}");
    println!("seed={seed}");
    println!("mode={:?}", args.mode);

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
