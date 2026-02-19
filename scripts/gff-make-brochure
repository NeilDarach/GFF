#!/usr/bin/env bash
set -eu
set -o pipefail
set -x
SCRIPT_DIR=$(dirname "$0")
TEMP_DIR=$(mktemp -d -t gffXXXXXX)
STATE_DIR="$1"

[[ ! -z "${TEMP_DIR}" ]] || exit 1
trap 'rm -rf -- "${TEMP_DIR}"' exit
if [[ -f "version.txt" ]]; then
    cp version.txt "${TEMP_DIR}"
else
    echo "1" >"${TEMP_DIR}/version.txt"
fi
cp "${SCRIPT_DIR}/../brochure"/*.typ "${TEMP_DIR}"
cp "${SCRIPT_DIR}/../brochure"/*.jpg "${TEMP_DIR}"
ln -s "${STATE_DIR}/posters" "${TEMP_DIR}/posters"

jq -L"${SCRIPT_DIR}" --slurpfile ref "${SCRIPT_DIR}/../brochure/ref.json" 'import "./gff" as gff; . | gff::generateSummary' showings.json >"${TEMP_DIR}/summary.json"
jq -L"${SCRIPT_DIR}" --slurpfile ref "${SCRIPT_DIR}/../brochure/ref.json" 'import "./gff" as gff; . | gff::generateBrochure' showings.json >"${TEMP_DIR}/brochure.json"
typst compile "${TEMP_DIR}/brochure.typ"
cp "${TEMP_DIR}/brochure.pdf" "${STATE_DIR}"
