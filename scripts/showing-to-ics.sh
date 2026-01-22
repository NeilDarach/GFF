echo "BEGIN:VCALENDAR"
echo "PRODID:-//GFFSLURP//v1//EN"
echo "VERSION:2.0"
echo "CALSCALE:GREGORIAN"
jq --slurpfile ref ./brochure/ref.json 'import "./scripts/gff" as gff; . | gff::generateIcalInfo' $1 |
    jq -r --slurpfile ref ./brochure/ref.json 'import "./scripts/gff" as gff; . | gff::generateIcal'
echo "END:VCALENDAR"

#time
#screenId
#movie.name
#movie.duration
