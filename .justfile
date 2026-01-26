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
    cd ./gff-fetch-summary ; node gff-fetch-summary.js > ../brochure/filter-summary.json
    cd ./brochure ; typst compile filter-summary.typ

publish-summaries:
    #!/usr/bin/env bash
    VER=$((1 + $(ssh root@goip.org.uk "ls -tr /opt/nginx/www/gff-2026-summaries-v*.pdf | tail -1 | sed -e 's/.*-v\(.\).pdf/\1/'")))
    echo ${VER} > ./brochure/summary-version.txt
    just summaries
    scp ./brochure/filter-summary.pdf "root@goip.org.uk:/opt/nginx/www/gff-2026-summaries-v${VER}.pdf"
    ssh root@goip.org.uk "ln -sf gff-2026-summaries-v${VER}.pdf /opt/nginx/www/gff-2026-summaries.pdf"

publish:
    #!/usr/bin/env bash
    VER=$((1 + $(ssh root@goip.org.uk "ls -tr /opt/nginx/www/gff-2026-v*.pdf | tail -1 | sed -e 's/.*-v\(.\).pdf/\1/'")))
    echo ${VER} > ./brochure/version.txt
    just brochure
    scp ./brochure/brochure.pdf "root@goip.org.uk:/opt/nginx/www/gff-2026-v${VER}.pdf"
    ssh root@goip.org.uk "ln -sf gff-2026-v${VER}.pdf /opt/nginx/www/gff-2026.pdf"
