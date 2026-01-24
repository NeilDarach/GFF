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

summaries:
    cd ./updateCalendar ; node filter-summary.js > ../brochure/filter-summary.json
    cd ./brochure ; typst compile filter-summary.typ

publish:
    just brochure
    scp ./brochure/brochure.pdf "root@goip.org.uk:/opt/nginx/www/gff-2026-v$(cat ./brochure/version.txt).pdf"
    ssh root@goip.org.uk "ln -sf gff-2026-v$(cat ./brochure/version.txt).pdf /opt/nginx/www/gff-2026.pdf"
    echo "$(( 1 + $(cat ./brochure/version.txt)))" > ./brochure/version.txt
