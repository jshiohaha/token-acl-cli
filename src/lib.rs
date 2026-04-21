mod cli;
mod commands;
mod rpc;
mod signer;

pub async fn run() -> anyhow::Result<()> {
    cli::run().await
}
