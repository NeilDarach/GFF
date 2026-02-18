#!/usr/bin/env bash
JSON="$1"
DIR=$(dirname "$1")
mkdir -p "${DIR}/posters"
for id in $(jq -r '(.. | .posterImage?)//""' "${JSON}" | sort -u); do
    if [[ ! -f "${DIR}/posters/${id}.jpg" ]]; then
        curl -s "https://indy-systems.imgix.net/${id}?fit=crop&w=400&h=600&fm=jpeg&auto=format,compress&cs=origin" >"${DIR}/posters/${id}.jpg"
    fi
done
