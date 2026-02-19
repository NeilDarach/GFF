#!/usr/bin/env bash
set -eu
set -o pipefail
set -x
SCRIPT_DIR=$(dirname "$0")
TEMP_DIR=$(mktemp -d -t gffXXXXXX)
STATE_DIR="$1"

[[ ! -z "${TEMP_DIR}" ]] || exit 1
trap 'rm -rf -- "${TEMP_DIR}"' exit
if [[ -f "summary-version.txt" ]]; then
    cp summary-version.txt "${TEMP_DIR}"
else
    echo "1" >"${TEMP_DIR}/summary-version.txt"
fi
cp "${SCRIPT_DIR}/../brochure"/*.typ "${TEMP_DIR}"
cp "${SCRIPT_DIR}/../brochure"/*.jpg "${TEMP_DIR}"
cp "${STATE_DIR}/summary.json" "${TEMP_DIR}"
cp "${STATE_DIR}/filter-summary.json" "${TEMP_DIR}"

typst compile "${TEMP_DIR}/filter-summary.typ"
cp "${TEMP_DIR}/filter-summary.pdf" "${STATE_DIR}"
