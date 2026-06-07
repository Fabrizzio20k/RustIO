#!/usr/bin/env bash
set -euo pipefail

PROFILE="pc"
TASK=""
for arg in "$@"; do
  case "$arg" in
    --profile=*) PROFILE="${arg#*=}" ;;
    *) TASK="$arg" ;;
  esac
done

MODEL_DIR="models"
MODEL_FILE="$MODEL_DIR/qwen3-4b-instruct-2507-q4_k_m.gguf"
MODEL_URL="https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
PORT=8080

case "$PROFILE" in
  rasp) SERVER_ARGS="-ngl 0 --threads 4 --parallel 1 -c 4096 --cache-ram 2048" ;;
  pc)   SERVER_ARGS="-ngl 99 -c 8192 --parallel 4" ;;
  *)    echo "perfil desconocido: $PROFILE (usa pc o rasp)"; exit 1 ;;
esac

mkdir -p "$MODEL_DIR"
if [ ! -f "$MODEL_FILE" ]; then
  curl -L -o "$MODEL_FILE" "$MODEL_URL"
fi

cargo build --release

if command -v llama-server >/dev/null 2>&1; then
  LLAMA_CMD="llama-server"
elif [ -x "./llama-server" ]; then
  LLAMA_CMD="./llama-server"
else
  echo "Error: llama-server no encontrado."
  exit 1
fi

$LLAMA_CMD -m "$MODEL_FILE" --port "$PORT" --jinja $SERVER_ARGS &
SERVER_PID=$!
trap 'kill "$SERVER_PID" 2>/dev/null || true' EXIT

until curl -sf "http://localhost:$PORT/health" >/dev/null 2>&1; do
  sleep 1
done

if [ -n "$TASK" ]; then
  ./target/release/rustio "$TASK"
else
  ./target/release/rustio
fi
