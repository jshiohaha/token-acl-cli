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

The CLI now exposes the broader Token ACL mint lifecycle commands from the main ACL client in addition to the gate-list helpers already in this example crate.

### `create-config`

Creates the Token ACL mint config PDA for a mint and optionally stores the gating program in the mint metadata.

```bash
gated-mint-cli --keypair <KEYPAIR> create-config --mint <MINT> [--gating-program <PROGRAM_ID>] [--freeze-authority <BASE58_OR_BYTES>]
```

Inputs:

- `--mint <MINT>`
  Mint address to configure.
- `--gating-program <PROGRAM_ID>`
  Optional gating program pubkey to store on the mint config and in mint metadata.
- `--freeze-authority <BASE58_OR_BYTES>`
  Optional freeze-authority signer in the same formats supported by `--keypair`.

Outputs:

- Prints `mint_config=<MINT_CONFIG_PDA>`.
- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

Example:

```bash
cargo run -- \
  --keypair '<KEYPAIR>' \
  create-config \
  --mint <MINT> \
  --gating-program <PROGRAM_ID>
```

### `delete-config`

Deletes a Token ACL mint config PDA and sends reclaimed lamports to the payer or an explicit receiver.

```bash
gated-mint-cli --keypair <KEYPAIR> delete-config --mint <MINT> [--receiver <RECEIVER>]
```

Inputs:

- `--mint <MINT>`
  Mint whose config PDA should be deleted.
- `--receiver <RECEIVER>`
  Optional lamport receiver. Defaults to the payer.

Outputs:

- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

### `set-authority`

Updates the authority on an existing mint config.

```bash
gated-mint-cli --keypair <KEYPAIR> set-authority --mint <MINT> --new-authority <NEW_AUTHORITY>
```

Inputs:

- `--mint <MINT>`
  Mint whose config authority should be updated.
- `--new-authority <NEW_AUTHORITY>`
  New authority pubkey.

Outputs:

- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

### `set-gating-program`

Updates the gating program on an existing mint config and mirrors the value into mint metadata.

```bash
gated-mint-cli --keypair <KEYPAIR> set-gating-program --mint <MINT> --new-gating-program <NEW_GATING_PROGRAM>
```

Inputs:

- `--mint <MINT>`
  Mint whose config should be updated.
- `--new-gating-program <NEW_GATING_PROGRAM>`
  New gating program pubkey.

Outputs:

- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

### `set-instructions`

Toggles permissionless thaw and freeze support on the mint config.

```bash
gated-mint-cli --keypair <KEYPAIR> set-instructions --mint <MINT> <--enable-thaw|--disable-thaw> <--enable-freeze|--disable-freeze>
```

Inputs:

- `--mint <MINT>`
  Mint whose permissionless instruction flags should be updated.
- `--enable-thaw` or `--disable-thaw`
  Required thaw toggle.
- `--enable-freeze` or `--disable-freeze`
  Required freeze toggle.

Outputs:

- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

Example:

```bash
cargo run -- \
  --keypair '<KEYPAIR>' \
  set-instructions \
  --mint <MINT> \
  --enable-thaw \
  --disable-freeze
```

### `freeze`

Freezes a token account using the configured ACL authority path.

```bash
gated-mint-cli --keypair <KEYPAIR> freeze --token-account <TOKEN_ACCOUNT>
```

Inputs:

- `--token-account <TOKEN_ACCOUNT>`
  Token-2022 account to freeze.

Outputs:

- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

### `freeze-permissionless`

Builds and sends the permissionless freeze path either from an existing token account or from a mint-owner pair.

```bash
gated-mint-cli --keypair <KEYPAIR> freeze-permissionless [--token-account <TOKEN_ACCOUNT> | (--mint <MINT> --owner <OWNER>)]
```

Inputs:

- `--token-account <TOKEN_ACCOUNT>`
  Existing token account to freeze.
- `--mint <MINT>`
  Mint address. Use with `--owner` when the token account should be derived as the ATA.
- `--owner <OWNER>`
  Token-account owner. Use with `--mint`.

Outputs:

- Prints:
  `mint=<MINT>`
  `token_account=<TOKEN_ACCOUNT>`
  `owner=<OWNER>`
- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

### `thaw`

Thaws a token account using the configured ACL authority path.

```bash
gated-mint-cli --keypair <KEYPAIR> thaw --token-account <TOKEN_ACCOUNT>
```

Inputs:

- `--token-account <TOKEN_ACCOUNT>`
  Token-2022 account to thaw.

Outputs:

- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

### `thaw-permissionless`

Builds and sends the permissionless thaw path either from an existing token account or from a mint-owner pair.

```bash
gated-mint-cli --keypair <KEYPAIR> thaw-permissionless [--token-account <TOKEN_ACCOUNT> | (--mint <MINT> --owner <OWNER>)]
```

Inputs:

- `--token-account <TOKEN_ACCOUNT>`
  Existing token account to thaw.
- `--mint <MINT>`
  Mint address. Use with `--owner` when the token account should be derived as the ATA.
- `--owner <OWNER>`
  Token-account owner. Use with `--mint`.

Outputs:

- Prints:
  `mint=<MINT>`
  `token_account=<TOKEN_ACCOUNT>`
  `owner=<OWNER>`
- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

Example:

```bash
cargo run -- \
  --keypair '<KEYPAIR>' \
  thaw-permissionless \
  --mint <MINT> \
  --owner <OWNER>
```

### `create-ata-and-thaw-permissionless`

Creates the Token-2022 associated token account for an owner and thaws it in one flow.

```bash
gated-mint-cli --keypair <KEYPAIR> create-ata-and-thaw-permissionless --mint <MINT> --owner <OWNER>
```

Inputs:

- `--mint <MINT>`
  Mint address.
- `--owner <OWNER>`
  Owner whose ATA should be created and thawed.

Outputs:

- Prints:
  `mint=<MINT>`
  `token_account=<TOKEN_ACCOUNT>`
  `owner=<OWNER>`
- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

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

Current behavior: this command name is historical and preserved for compatibility. It currently thaws a token account; prefer the explicit `thaw` command for new usage.

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

### `create-list`

Creates a Token ACL Gate list config PDA for the current signer.

```bash
gated-mint-cli --keypair <KEYPAIR> create-list [--mode <MODE>] [--seed <SEED>]
```

Inputs:

- `--mode <MODE>`
  List mode.
  Possible values: `allow`, `allow-all-eoas`, `block`
  Default: `allow`
- `--seed <SEED>`
  Optional deterministic seed pubkey. If omitted, the CLI generates a fresh random seed.

Outputs:

- Prints:
  `list_config=<LIST_CONFIG>`
  `seed=<SEED>`
  `mode=<MODE>`
- With `--simulate`:
  Prints `simulation=<...>`.
- Without `--simulate`:
  Prints `transaction=<explorer-url>`.

Example:

```bash
cargo run -- \
  --keypair '<KEYPAIR>' \
  create-list \
  --mode allow
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
