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

echo "Retrieving model"
ollama pull $OLLAMA_MODEL
echo "Done."

# Wait for Ollama process to finish.
wait $pid
