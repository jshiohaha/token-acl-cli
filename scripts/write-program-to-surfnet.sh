#!/usr/bin/env bash
set -euo pipefail

PROGRAM_SO="${1:-}"
AUTHORITY="${2:-}"
RPC_URL="${RPC_URL:-http://127.0.0.1:8899}"
PROGRAM_ID="${PROGRAM_ID:-GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz}"
CHUNK_BYTES="${CHUNK_BYTES:-900}"
RPC_RETRIES="${RPC_RETRIES:-20}"
RPC_RETRY_DELAY_SECS="${RPC_RETRY_DELAY_SECS:-0.25}"

if [[ -z "${PROGRAM_SO}" ]]; then
  echo "usage: $0 <program.so> [authority-pubkey]"
  echo "env:"
  echo "  RPC_URL=http://127.0.0.1:8899"
  echo "  PROGRAM_ID=GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz"
  echo "  CHUNK_BYTES=900"
  exit 1
fi

if [[ ! -f "${PROGRAM_SO}" ]]; then
  echo "program file not found: ${PROGRAM_SO}" >&2
  exit 1
fi

require_cmd() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "missing required command: ${cmd}" >&2
    exit 1
  fi
}

require_cmd curl
require_cmd jq
require_cmd xxd

rpc() {
  local payload="$1"
  local response
  response="$(
    curl -sS \
      --fail \
      -H 'Content-Type: application/json' \
      -d "${payload}" \
      "${RPC_URL}"
  )"

  if [[ "$(jq -r 'has("error")' <<<"${response}")" == "true" ]]; then
    echo "rpc error: $(jq -c '.error' <<<"${response}")" >&2
    return 1
  fi

  printf '%s\n' "${response}"
}

wait_for_rpc() {
  local attempt
  for ((attempt = 1; attempt <= RPC_RETRIES; attempt++)); do
    if curl -sS --fail -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
      "${RPC_URL}" >/dev/null; then
      return 0
    fi
    sleep "${RPC_RETRY_DELAY_SECS}"
  done

  echo "rpc did not become reachable at ${RPC_URL} after ${RPC_RETRIES} attempts" >&2
  exit 1
}

echo "waiting for rpc at ${RPC_URL}"
wait_for_rpc

if [[ -n "${AUTHORITY}" ]]; then
  echo "setting Surfnet program authority for ${PROGRAM_ID} to ${AUTHORITY}"
  authority_response="$(
    rpc "$(printf '{"jsonrpc":"2.0","id":1,"method":"surfnet_setProgramAuthority","params":["%s","%s"]}' "${PROGRAM_ID}" "${AUTHORITY}")"
  )"
  echo "${authority_response}"
  echo
else
  echo "no authority provided; relying on Surfpool default authority handling for writes"
fi

offset=0
chunk_index=0

while IFS= read -r hex_chunk; do
  [[ -z "${hex_chunk}" ]] && continue
  chunk_len_bytes=$((${#hex_chunk} / 2))
  chunk_index=$((chunk_index + 1))

  echo "writing chunk ${chunk_index} at offset ${offset} (${chunk_len_bytes} bytes)"
  if [[ -n "${AUTHORITY}" ]]; then
    rpc "$(printf '{"jsonrpc":"2.0","id":1,"method":"surfnet_writeProgram","params":["%s","%s",%d,"%s"]}' "${PROGRAM_ID}" "${hex_chunk}" "${offset}" "${AUTHORITY}")" >/dev/null
  else
    rpc "$(printf '{"jsonrpc":"2.0","id":1,"method":"surfnet_writeProgram","params":["%s","%s",%d]}' "${PROGRAM_ID}" "${hex_chunk}" "${offset}")" >/dev/null
  fi

  offset=$((offset + chunk_len_bytes))
done < <(xxd -p -c "${CHUNK_BYTES}" "${PROGRAM_SO}")

echo "wrote ${offset} bytes to ${PROGRAM_ID} on ${RPC_URL}"
