use anyhow::{Result, bail};
use clap::{Args, ValueEnum};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::{Keypair, Signer};
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use token_acl_gate_client::{
    accounts::ListConfig,
    instructions::{
        // SetupFreezeExtraMetasBuilder, SETUP_FREEZE_EXTRA_METAS_DISCRIMINATOR
        CreateListBuilder,
        SETUP_EXTRA_METAS_DISCRIMINATOR,
        SetupExtraMetasBuilder,
    },
    programs::TOKEN_ACL_GATE_PROGRAM_ID,
    types::Mode,
};

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
    #[arg(long)]
    pub mint: Pubkey,
    #[arg(long, value_enum, default_value_t = ListModeArg::Allow)]
    pub mode: ListModeArg,
    #[arg(long)]
    pub seed: Option<Pubkey>,
}

pub async fn run(ctx: &AppContext, args: CreateListArgs) -> Result<()> {
    let authority = ctx.payer.pubkey();
    let seed = args.seed.unwrap_or_else(|| Keypair::new().pubkey());
    let list_config = ListConfig::find_pda(&authority, &seed).0;
    let mint_config = token_acl_client::accounts::MintConfig::find_pda(&args.mint).0;
    let gate_program_id = TOKEN_ACL_GATE_PROGRAM_ID;
    let thaw_extra_metas =
        token_acl_interface::get_thaw_extra_account_metas_address(&args.mint, &gate_program_id);
    // let freeze_extra_metas =
    //     token_acl_interface::get_freeze_extra_account_metas_address(&args.mint, &gate_program_id);

    let mut create_list_builder = CreateListBuilder::new();
    create_list_builder
        .authority(authority)
        .payer(authority)
        .seed(seed)
        .mode(args.mode.into())
        .list_config(list_config);

    let remaining_accounts = [AccountMeta::new_readonly(list_config, false)];

    let mut setup_thaw_builder = SetupExtraMetasBuilder::new();
    setup_thaw_builder
        .authority(authority)
        .payer(authority)
        .token_acl_mint_config(mint_config)
        .mint(args.mint)
        .extra_metas(thaw_extra_metas)
        .add_remaining_accounts(&remaining_accounts);

    // let mut setup_freeze_builder = SetupFreezeExtraMetasBuilder::new();
    // setup_freeze_builder
    //     .authority(authority)
    //     .payer(authority)
    //     .token_acl_mint_config(mint_config)
    //     .mint(args.mint)
    //     .extra_metas(freeze_extra_metas)
    //     .add_remaining_accounts(&remaining_accounts);

    let create_list_ix = create_list_builder.instruction();
    let setup_thaw_ix = setup_thaw_builder.instruction();
    // let setup_freeze_ix = setup_freeze_builder.instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_list_ix, setup_thaw_ix], // setup_freeze_ix
        Some(&authority),
        &[ctx.payer.as_ref()],
        ctx.rpc_client.get_latest_blockhash().await?,
    );

    println!("mint={}", args.mint);
    println!("mint_config={mint_config}");
    println!("list_config={list_config}");
    println!("thaw_extra_metas={thaw_extra_metas}");
    // println!("freeze_extra_metas={freeze_extra_metas}");
    println!("seed={:?}", seed.to_string());
    println!("mode={:?}", args.mode);

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
