# gated-mint-cli

Rust CLI for working with gated Token-2022 mints and Token ACL Gate lists on Solana.

## Build

```bash
cargo build
```

## Global Flags

```bash
gated-mint-cli --keypair <BASE58_OR_BYTES> [--rpc-url <RPC_URL>] [--simulate] <COMMAND>
```

### Inputs

- `--keypair <BASE58_OR_BYTES>`
  Required signer input.
- `--rpc-url <RPC_URL>`
  Optional RPC endpoint.
  Default: `https://orca.rpcpool.com/ae9a156f6bdd344c8267465eb432`
- `--simulate`
  Simulates transactions instead of sending them.

### Supported `--keypair` Formats

- Base58 secret key string.
- `32` secret-key bytes as a comma-separated list.
- `64` serialized keypair bytes as a comma-separated list.
- Byte lists may be wrapped in brackets, for example:
  `[1,2,3,...]`

Examples:

```bash
gated-mint-cli --keypair '3Q4x...'
```

```bash
gated-mint-cli --keypair '[12,34,56,...]'
```

## Commands

### `create-mint`

Creates a Token-2022 mint configured for Token ACL gate usage.

```bash
gated-mint-cli --keypair <KEYPAIR> create-mint [OPTIONS]
```

Inputs:

- `--name <NAME>`
  Token name.
  Default: `That's a nice earf`
- `--symbol <SYMBOL>`
  Token symbol.
  Default: `EARF`
- `--uri <URI>`
  Metadata URI.
- `--gate-program-id <GATE_PROGRAM_ID>`
  Gate program id.
  Default: `GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz`
- `--transfer-fee-basis-points <TRANSFER_FEE_BASIS_POINTS>`
  Transfer fee in basis points.
  Default: `100`
- `--maximum-transfer-fee <MAXIMUM_TRANSFER_FEE>`
  Maximum transfer fee in base units.
  Default: `100000000`
- `--decimals <DECIMALS>`
  Mint decimals.
  Default: `6`

Outputs:

- Prints payer address and balance before mint creation.
- Prints the newly generated mint address.
- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

Example:

```bash
cargo run -- \
  --keypair '<KEYPAIR>' \
  create-mint \
  --name 'Example Token' \
  --symbol EXMPL \
  --uri 'https://example.com/token.json'
```

### `mint`

Current behavior: this command name is historical. It currently thaws a token account; it does not mint new tokens.

```bash
gated-mint-cli --keypair <KEYPAIR> mint --mint <MINT> --owner <OWNER> --token-account <TOKEN_ACCOUNT>
```

Inputs:

- `--mint <MINT>`
  Mint address.
- `--owner <OWNER>`
  Owner address.
  Currently informational only for output.
- `--token-account <TOKEN_ACCOUNT>`
  Token account to thaw.

Outputs:

- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `owner=<OWNER>`.
  Prints `transaction=<explorer-url>`.

Example:

```bash
cargo run -- \
  --keypair '<KEYPAIR>' \
  mint \
  --mint <MINT> \
  --owner <OWNER> \
  --token-account <TOKEN_ACCOUNT>
```

### `close-wallet-entries`

Fetches all wallet-entry accounts for a given list config, batches `RemoveWallet` instructions into transactions, and sends or simulates them.

```bash
gated-mint-cli --keypair <KEYPAIR> close-wallet-entries --list-config <LIST_CONFIG> [--batch-size <BATCH_SIZE>]
```

Inputs:

- `--list-config <LIST_CONFIG>`
  Token ACL Gate list config account.
- `--batch-size <BATCH_SIZE>`
  Number of remove-wallet instructions per transaction.
  Default: `8`

Behavior:

- Scans all `WalletEntry` accounts owned by the gate program for the provided `list_config`.
- Decodes each entry.
- Builds `RemoveWallet` instructions.
- Chunks them into transactions.

Outputs:

- Prints:
  `list_config=<LIST_CONFIG>, wallet_entries=<COUNT>, batch_size=<BATCH_SIZE>`
- With `--simulate`:
  Prints `batch=<N> simulation=<...>` for each batch.
- Without `--simulate`:
  Prints `batch=<N> transaction=<explorer-url>` for each batch.

Example:

```bash
cargo run -- \
  --keypair '<KEYPAIR>' \
  close-wallet-entries \
  --list-config <LIST_CONFIG> \
  --batch-size 8
```

### `delete-list`

Deletes a list config account using the current signer as authority.

```bash
gated-mint-cli --keypair <KEYPAIR> delete-list --list-config <LIST_CONFIG>
```

Inputs:

- `--list-config <LIST_CONFIG>`
  Token ACL Gate list config account to delete.

Behavior:

- Fetches and decodes the list config account.
- Verifies the signer matches the decoded `authority`.
- Builds a `DeleteList` instruction.
- Sends or simulates one transaction.

Outputs:

- Prints:
  `list_config=<LIST_CONFIG>, seed=<SEED>, mode=<MODE>, wallets_count=<COUNT>`
- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

Example:

```bash
cargo run -- \
  --keypair '<KEYPAIR>' \
  delete-list \
  --list-config <LIST_CONFIG>
```

## Notes

- `--simulate` is the safest way to verify the instruction set before sending transactions.
- `delete-list` does a local authority check, but on-chain program rules still apply.
  For example, if the program requires an empty list before deletion, the transaction can still fail on-chain.
