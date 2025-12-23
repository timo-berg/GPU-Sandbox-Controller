#!/usr/bin/env bash
set -euo pipefail

URL="${URL:-http://127.0.0.1:3000/jobs/list}"
INTERVAL="${INTERVAL:-1}"

while true; do
  clear
  curl -s "$URL" \
    | jq -r '.jobs[] | "\(.job_id)  tenant=\(.tenant_id)  status=\(.status)"'
  sleep "$INTERVAL"
done