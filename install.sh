mkdir -p models
curl -L -o models/qwen2.5-coder-7b-instruct-q4_k_m.gguf https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/main/qwen2.5-coder-7b-instruct-q4_k_m.gguf
llama-server -m models/qwen2.5-coder-7b-instruct-q4_k_m.gguf -ngl 99 -c 8192 --port 8080 --jinja --chat-template-file qwen2.5-tool-template.jinja