#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$ModelDir = "models"
$ModelFile = Join-Path $ModelDir "qwen3-4b-instruct-2507-q4_k_m.gguf"
$ModelUrl = "https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
$Port = 8080

New-Item -ItemType Directory -Force -Path $ModelDir | Out-Null
if (-not (Test-Path $ModelFile)) {
    curl.exe -L -o $ModelFile $ModelUrl
}

llama-server --model $ModelFile --port $Port
