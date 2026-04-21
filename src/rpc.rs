use anyhow::Result;
use solana_pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

pub const LOCALHOST_RPC_URL: &str = "http://127.0.0.1:8899";

pub async fn airdrop_if_localnet(
    rpc_client: &RpcClient,
    rpc_url: &str,
    pubkey: &Pubkey,
    lamports: u64,
) -> Result<()> {
    if rpc_url != LOCALHOST_RPC_URL {
        return Ok(());
    }

    let airdrop_signature = rpc_client.request_airdrop(pubkey, lamports).await?;
    loop {
        if rpc_client.confirm_transaction(&airdrop_signature).await? {
            break;
        }
    }

    Ok(())
}

pub fn explorer_tx_url(rpc_url: &str, signature: &str) -> String {
    if rpc_url == LOCALHOST_RPC_URL {
        return format!(
            "https://explorer.solana.com/tx/{signature}?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899"
        );
    }

    format!("https://explorer.solana.com/tx/{signature}")
}
