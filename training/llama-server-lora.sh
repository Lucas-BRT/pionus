#!/bin/sh
# Stands in for llama-server (PIONUS_LLAMA_SERVER) to apply a LoRA
# adapter on top of the GGUF pionus loads, so a fine-tune is judged
# on the upstream quantized weights instead of a re-quantized merge.
#
#   PIONUS_LLAMA_SERVER=training/llama-server-lora.sh \
#   LORA_SERVER=~/.local/share/pionus/llama.cpp/llama-b11381/llama-server \
#   LORA_GGUF=runs/h1/adapter.gguf pionus eval ...
set -eu
: "${LORA_SERVER:?path of the real llama-server}"
: "${LORA_GGUF:?path of the GGUF adapter}"
exec "$LORA_SERVER" "$@" --lora "$LORA_GGUF"
