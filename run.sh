MODEL_DIR="models"
MODEL_FILE="$MODEL_DIR/qwen3-4b-instruct-2507-q4_k_m.gguf"
MODEL_URL="https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
PORT=8080
CTX_SIZE=8192


mkdir -p "$MODEL_DIR"
if [ ! -f "$MODEL_FILE" ]; then
  curl -L -o "$MODEL_FILE" "$MODEL_URL"
fi

llama-server --model "$MODEL_FILE" --port "$PORT" --ctx-size "$CTX_SIZE" \
  --jinja \
  --threads 4 -fa on \
  --cache-type-k q8_0 --cache-type-v q8_0 \
  --mlock --cont-batching