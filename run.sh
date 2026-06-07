MODEL_DIR="models"
MODEL_FILE="$MODEL_DIR/qwen3-4b-instruct-2507-q4_k_m.gguf"
MODEL_URL="https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
PORT=8080


mkdir -p "$MODEL_DIR"
if [ ! -f "$MODEL_FILE" ]; then
  curl -L -o "$MODEL_FILE" "$MODEL_URL"
fi

llama-server --model "$MODEL_FILE" --port "$PORT"