#!/bin/bash

API_URL="http://localhost:$1/alerts"
OUTPUT_FILE="alerts.jsonl"

truncate -s0 "$OUTPUT_FILE"

while true; do
    # Unix timestamp in seconds
    TIMESTAMP=$(date +%s)

    # Fetch API response
    RESPONSE=$(curl -s "$API_URL")

    # Inject timestamp into JSON using jq
    echo "$RESPONSE" | jq -c --argjson ts "$TIMESTAMP" '. + {timestamp: $ts}' >> "$OUTPUT_FILE"

    # Wait for some time
    sleep "$2"
done
