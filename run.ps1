#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$ModelDir = "models"
$ModelFile = Join-Path $ModelDir "qwen3.5-2b-q4_k_m.gguf"
$ModelUrl = "https://huggingface.co/unsloth/Qwen3.5-2B-GGUF/resolve/main/Qwen3.5-2B-Q4_K_M.gguf"
$Port = 8080
$CtxSize = 8192

New-Item -ItemType Directory -Force -Path $ModelDir | Out-Null
if (-not (Test-Path $ModelFile)) {
    curl.exe -L -o $ModelFile $ModelUrl
}

llama-server --model $ModelFile --port $Port --ctx-size $CtxSize `
    --jinja `
    --threads 1 -fa on `
    --cache-type-k q8_0 --cache-type-v q8_0 `
    --mlock --cont-batching
