default:
    echo "Not a command""

rebuild:
    ./getIds.sh > ids.json
    cat ids.json | jq -r keys[] | xargs -n 1 ./get_showings.sh | jq -s . > showings.json
    ./showing-to-csv.sh 
    just posters
    just brochure
    just summary

brochure:
    jq --slurpfile ref ref.json 'import "gff" as gff; . | gff::generateBrochure ' showings.json > brochure/brochure.json
    cd brochure; typst compile brochure.typ

summary:
    jq --slurpfile ref ref.json 'import "gff" as gff; . | gff::generateSummary ' showings.json > summary/summary.json
    cd summary; typst compile summary.typ

posters:
    ./fetchImages.sh
