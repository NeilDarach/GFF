default:
    @just --list

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
    curl https://goip.org.uk/gff/summary > ./brochure/filter-summary.json
    cd ./brochure ; typst compile filter-summary.typ

publish-summaries:
    #!/usr/bin/env bash
    VER=$((1 + $(ssh goip.org.uk "bash -c 'ls -tr /strongStateDir/nginx/gff-2026-summaries-v?*.pdf' | tail -1 | sed -e 's/.*-v\(.\+\).pdf/\1/'")))
    echo ${VER} > ./brochure/summary-version.txt
    just summaries
    cat ./brochure/filter-summary.pdf | ssh goip.org.uk "sudo bash -c 'cat > /strongStateDir/nginx/gff-2026-summaries-v${VER}.pdf'"
    ssh goip.org.uk "sudo ln -sf gff-2026-summaries-v${VER}.pdf /strongStateDir/nginx/gff-2026-summaries.pdf"

publish:
    #!/usr/bin/env bash
    VER=$((1 + $(ssh goip.org.uk "bash -c 'ls -tr /strongStateDir/nginx/gff-2026-v?*.pdf' | tail -1 | sed -e 's/.*-v\(.\+\).pdf/\1/'")))
    echo ${VER} > ./brochure/version.txt
    just brochure
    cat ./brochure/brochure.pdf | ssh goip.org.uk "sudo bash -c 'cat > /strongStateDir/nginx/gff-2026-v${VER}.pdf'"
    ssh goip.org.uk "sudo ln -sf gff-2026-v${VER}.pdf /strongStateDir/nginx/gff-2026.pdf"
