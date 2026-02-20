#!/usr/bin/env bash
JSON="$1"
STATE_IR=$(dirname "$1")
mkdir -p "${STATE_DIR}/posters"
for id in $(jq -r '(.. | .posterImage?)//""' "${JSON}" | sort -u); do
    if [[ ! -f "${STATE_DIR}/posters/${id}.jpg" ]]; then
        curl -s "https://indy-systems.imgix.net/${id}?fit=crop&w=400&h=600&fm=jpeg&auto=format,compress&cs=origin" >"${STATE_DIR}/posters/${id}.jpg"
    fi
done
