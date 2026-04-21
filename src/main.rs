#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gated_mint_cli::run().await
}
