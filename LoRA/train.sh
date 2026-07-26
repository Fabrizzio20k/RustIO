#!/bin/bash

uv pip install -r requirements.txt

mlx_lm.lora \
  --model Qwen/Qwen3.5-0.8B \
  --train \
  --data . \
  --iters 500 \
  --batch-size 2 \
  --num-layers 4

# 2. Fusionar los pesos y exportar en formato crudo (safetensors)
mlx_lm.fuse \
  --model Qwen/Qwen3.5-0.8B \
  --adapter-path adapters \
  --save-path fused_model

# 5. Convertir a GGUF y mover a la carpeta models
python llama.cpp/convert_hf_to_gguf.py fused_model --outfile ../models/modelo_tuneado.gguf --outtype q8_0
