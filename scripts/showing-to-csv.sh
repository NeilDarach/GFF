# #!/usr/bin/env bash
# Subject, Start Date, Start Time, End Time, All Day Event, Description, Location
# "I Am Martin Parr", "02-25-2025", "1:10 PM","2:18 PM", False, "", "GFT 1"
DIR=$(dirname "$0")
JSON="$1"

jq --slurpfile ref "${DIR}/../brochure/ref.json" "import \"${DIR}/gff\" as gff; . | gff::generateCsvInfo" "${JSON}" |
    jq -r --slurpfile ref "${DIR}/../brochure/ref.json" "import \"${DIR}/gff\" as gff; . | gff::generateCsv"
