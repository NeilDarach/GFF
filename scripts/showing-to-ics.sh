#!/usr/bin/env bash
DIR=$(dirname "$0")
JSON="$1"
echo "BEGIN:VCALENDAR"
echo "PRODID:-//GFFSLURP//v1//EN"
echo "VERSION:2.0"
echo "CALSCALE:GREGORIAN"
jq --slurpfile ref "${DIR}/../brochure/ref.json" "import \"${DIR}/gff\" as gff; . | gff::generateIcalInfo" "${JSON}" |
    jq -r --slurpfile ref "${DIR}/../brochure/ref.json" "import \"${DIR}/gff\" as gff; . | gff::generateIcal"
echo "END:VCALENDAR"

#time
#screenId
#movie.name
#movie.duration
