#!/bin/bash

if [ -f /zscaler/zscaler.crt ]; then
  echo "Adding Zscaler root certificate..."
  cp /zscaler/zscaler.crt /usr/local/share/ca-certificates/zscaler.crt
  update-ca-certificates
fi

# Start Ollama in the background.
/bin/ollama serve &
# Record Process ID.
pid=$!

# Pause for Ollama to start.
sleep 5

# Get models list from environment variable
IFS=',' read -ra MODELS <<<"$OLLAMA_MODEL"

echo "Retrieving models..."
for MODEL in "${MODELS[@]}"; do
  echo "Pulling model: $MODEL"
  ollama pull $MODEL
  echo "Model $MODEL downloaded successfully."
done
echo "All models downloaded."

# Wait for Ollama process to finish.
wait $pid
