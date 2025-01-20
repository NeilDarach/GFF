rebuild:
    ./getIds.sh > ids.json
    cat ids.json | jq -r keys[] | xargs -n 1 ./get_showings.sh | jq -s . > showings.json
    ./fetchImages.sh
    jq --slurpfile ref ref.json 'import "gff" as gff; . | gff::generateBrochure ' showings.json | jq -s "." > brochure/brochure.json
    cd brochure; typst compile brochure.typ
    ./showing-to-csv.sh 
    jq --slurpfile ref ref.json 'import "gff" as gff; . | gff::generateSummary ' showings.json > summary/summary.json
    cd summary; typst compile summary.typ
