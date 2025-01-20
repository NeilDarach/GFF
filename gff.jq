def getScreen:
  #expect screenID - 228
  #return "Cottiers"
  ($ref[].screens[.] // "missing")
  ;

def getStrand:
  #expect badge string - [ "853", "828", "549", "583" ]
  #returns the first which matches a strand -  "Our Story So Far"
  ((.[] | $ref[].strands[.])//"") 
  ;

def sheetHeaders: 
  [ "Title","Time","Duration (min)","Screen","Synopsis","Staring","Genre","All Genres","Directed By","Rating","Rating Reason","Strand","Youtube Trailer Id","Poster" ]
  ;

def icalHeaders:
  [ "BEGIN", "DTSTART", "DTEND", "LOCATION", "SUMMARY", "END" ]
  ;

def calendarHeaders:
  [ "Subject", "Start Date", "All Day Event", "Start Time", "End Time", "Location", "Description", "Private" ]
  ;

def calendarDate: 
  # expect 2024-03-08T19:45:00Z
  # return 03/08/2024
  fromdate | strftime("%m/%d/%Y")
  ;

def icalStart:
  fromdate | strftime("%y%m%dT%H%M%SZ")
  ;

def calendarTime:
  # expect 2024-03-08T19:45:00Z
  # return 7:45pm
  fromdate | strftime("%l:%M %p") | ltrimstr(" ")
  ;

def icalEnd:
  .movie.duration as $duration | .time | fromdate + ($duration * 60) | todate | icalStart
;


def sortName:
  if startswith("The ") then (.[4:]+", The") 
  else if startswith("A ") then (.[2:]+", A") 
  else if startswith("An ") then (.[3:]+", An") 
  else . end end end
  ;

def calendarEndTime:
  # expect 2024-03-08T19:45:00Z
  # take 30
  # return 8:15pm
  .movie.duration as $duration | .time | fromdate + ($duration * 60) | todate | calendarTime
  ;

def calendarTime(duration):
  # expect 2024-03-08T19:45:00Z
  # take 30
  # return 8:15pm
  fromdate + (duration * 60) | todate | calendarTime
  ;

def sheetDateTime:
  # expect 2024-03-08T19:45:00Z
  # return 2024/03/08 19:45
  fromdate | strftime("%Y/%m/%d %H:%M")
  ;

def brochureTime:
  fromdate | strftime("%k:%M") | ltrimstr(" ")
  ;

def brochureDate: 
  fromdate | strftime("%a, %B ") + (strftime("%d") |  ltrimstr("0"))
  ;

def youtubeUrl:
  "https://www.youtube.com/watch?v=" + .
  ;

def posterUrl:
  "https://indy-systems.imgix.net/" + .
  ;
   

def posterFile:
  "posters/" + . + ".jpg"
  ;
   

def processShowings: 
  [ .movie.name, (.time|sheetDateTime), .movie.duration, (.screenId|getScreen), (.movie.synopsis), (.movie.starring//""), (.movie.genre//""), (.movie.allGenres//""), (.movie.directedBy//""),(.movie.rating), (.movie.ratingReason//""), (.showingBadgeIds|getStrand), (.movie.trailerYoutubeId|youtubeUrl//""),(.movie.posterImage|posterUrl)]
  ;

def generateCalendar:
  [ .movie.name, (.time|calendarDate),"False",(.time|calendarTime),calendarEndTime,(.screenId|getScreen),"","False" ]
  ;

def combineGenres:
   .genre as $g | ([$g] + (.allGenres//"" | split(", ") | map(select(. != $g)))//[]) | join(", ")
  ;

def tr:
  rtrimstr("\n")
  ;

def stripSynopsis:
  gsub("______*";"____";"m")|gsub("</?i>";"_";"m")|gsub("</?b>";"*";"m")|gsub("<a .*</a>";"";"m")|gsub("</style>";"";"m")|gsub("</?font[^>]+>";"";"m")|tr
  ;

def generateBrochure:
  group_by(.movie.id)[] | { "name": (.[0].movie.name), "sortname": (.[0].movie.name|sortName), "showings": [.[] | { "screen": (.screenId|getScreen), "time": (.time|brochureTime), "date":(.time|brochureDate), "datetime":.time} ], "duration": .[0].movie.duration, "synopsis": (first.movie.synopsis|stripSynopsis), "starring": (first.movie.starring//""|tr) , "genres": (first.movie | combineGenres), "directedBy":(first.movie.directedBy//""|tr) , "rating":(first.movie.rating|tr), "ratingReason":(first.movie.ratingReason//""|tr), "strand":(first.showingBadgeIds|getStrand), "poster":(first.movie.posterImage|posterFile) }
  ;

def generateCsvInfo:
  [ .[] | { "Subject": .movie.name, "Start Date": (.time|calendarDate), "Start Time": (.time|calendarTime), "End Time": (.|calendarEndTime), "All Day Event":"False", "Description":"", "Location": (.screenId|getScreen), "Private": "False" } ]
  ;

def  generateCsv:
  map (. as $row | calendarHeaders | map($row[.])) as $rows | calendarHeaders, $rows[] | @csv
  ;

def generateIcalInfo:
  .[] | "BEGIN:VEVENT\nSUMMARY:\(.movie.name)\nDTSTART:\(.time|icalStart)\nDTEND:\(.|icalEnd)\nLOCATION:\(.screenId|getScreen)\nEND:VEVENT"
  ;

def  generateIcal:
  .
  ;

def generateSummary:
  [ .[] | {"date":.time[:10], "screen":.screenId, "start": .time[11:16],"title": .movie.name, "duration":.movie.duration, "color":"xFFFFFF" }] | sort_by(.screen)|group_by(.date) | map({"key": .[0].date, value: (map(.)|group_by(.screen)|map({"key":(.[0].screen|getScreen), value: map({"start":.start,"title":.title,"duration":.duration,"color":"ffffdd"})}))|from_entries}) | from_entries
  ;

