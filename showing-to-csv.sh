# Subject, Start Date, Start Time, End Time, All Day Event, Description, Location
# "I Am Martin Parr", "02-25-2025", "1:10 PM","2:18 PM", False, "", "GFT 1"

jq --slurpfile ref ref.json 'import "gff" as gff; . | gff::generateCsvInfo' showings.json  \
 |  jq -r --slurpfile ref ref.json 'import "gff" as gff; . | gff::generateCsv'  > showings-for-gcal.csv

echo "BEGIN:VCALENDAR" > showings-for-gcal.ics
echo "PRODID:-//GFFSLURP//v1//EN" >> showings-for-gcal.ics
echo "VERSION:2.0" >> showings-for-gcal.ics
echo "CALSCALE:GREGORIAN" >> showings-for-gcal.ics
jq --slurpfile ref ref.json 'import "gff" as gff; . | gff::generateIcalInfo' showings.json  \
 |  jq -r --slurpfile ref ref.json 'import "gff" as gff; . | gff::generateIcal'  >> showings-for-gcal.ics
echo "END:VCALENDAR" >> showings-for-gcal.ics

#time
#screenId
#movie.name
#movie.duration

