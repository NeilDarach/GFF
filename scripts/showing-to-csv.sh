# Subject, Start Date, Start Time, End Time, All Day Event, Description, Location
# "I Am Martin Parr", "02-25-2025", "1:10 PM","2:18 PM", False, "", "GFT 1"

jq --slurpfile ref ./brochure/ref.json 'import "./scripts/gff" as gff; . | gff::generateCsvInfo' $1 |
    jq -r --slurpfile ref ./brochure/ref.json 'import "./scripts/gff" as gff; . | gff::generateCsv'
