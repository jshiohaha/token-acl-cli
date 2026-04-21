use anyhow::{Result, anyhow, bail};
use solana_keypair::Keypair;

pub fn parse_keypair_arg(value: &str) -> Result<Keypair> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        bail!("keypair argument cannot be empty");
    }

    if looks_like_byte_list(trimmed) {
        return parse_keypair_bytes(trimmed);
    }

    parse_base58_keypair(trimmed)
}

fn looks_like_byte_list(value: &str) -> bool {
    value.starts_with('[') || value.contains(',')
}

fn parse_keypair_bytes(value: &str) -> Result<Keypair> {
    let normalized = value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();

    let bytes = normalized
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<u8>()
                .map_err(|err| anyhow!("invalid keypair byte '{part}': {err}"))
        })
        .collect::<Result<Vec<_>>>()?;

    match bytes.len() {
        32 => {
            let secret_key: [u8; 32] = bytes
                .try_into()
                .map_err(|_| anyhow!("invalid 32-byte secret key array"))?;
            Ok(Keypair::new_from_array(secret_key))
        }
        64 => {
            let slice: &[u8] = bytes.as_slice();
            Keypair::try_from(slice).map_err(|err| anyhow!("invalid 64-byte keypair array: {err}"))
        }
        len => bail!("keypair byte array must contain 32 or 64 bytes, found {len}"),
    }
}

fn parse_base58_keypair(value: &str) -> Result<Keypair> {
    std::panic::catch_unwind(|| Keypair::from_base58_string(value))
        .map_err(|_| anyhow!("invalid base58 keypair string"))
}
