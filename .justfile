default:
    echo "Not a command""

rebuild:
    ./scripts/getIds.sh > ./brochure/ids.json
    cat ./brochure/ids.json | jq -r keys[] | xargs -n 1 ./scripts/get_showings.sh | jq -s . > ./brochure/showings.json
    ./scripts/showing-to-csv.sh ./brochure/showings.json > ./brochure/showings.csv
    ./scripts/showing-to-ics.sh ./brochure/showings.json > ./brochure/showings.ics
    ./scripts/fetchImages.sh ./brochure/showings.json
    just brochure

brochure:
    jq --slurpfile ref ./brochure/ref.json 'import "./scripts/gff" as gff; . | gff::generateSummary ' ./brochure/showings.json > brochure/summary.json
    jq --slurpfile ref ./brochure/ref.json 'import "./scripts/gff" as gff; . | gff::generateBrochure ' ./brochure/showings.json > brochure/brochure.json
    cd brochure; typst compile summary.typ; typst compile brochure.typ

