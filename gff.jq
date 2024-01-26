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

def calendarHeaders:
  [ "Subject", "Start Date", "All Day Event", "Start Time", "End Time", "Location", "Description", "Private" ]
  ;

def calendarDate: 
  # expect 2024-03-08T19:45:00Z
  # return 03/08/2024
  fromdate | strftime("%m/%d/%Y")
  ;

def calendarTime:
  # expect 2024-03-08T19:45:00Z
  # return 7:45pm
  fromdate | strftime("%l:%M %p") | ltrimstr(" ")
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

def youtubeUrl:
  "https://www.youtube.com/watch?v=" + .
  ;

def posterUrl:
  "https://indy-systems.imgix.net/" + .
  ;
   

def processShowings: 
  [ .movie.name, (.time|sheetDateTime), .movie.duration, (.screenId|getScreen), (.movie.synopsis), (.movie.starring//""), (.movie.genre//""), (.movie.allGenres//""), (.movie.directedBy//""),(.movie.rating), (.movie.ratingReason//""), (.showingBadgeIds|getStrand), (.movie.trailerYoutubeId|youtubeUrl//""),(.movie.posterImage|posterUrl)]
  ;

def generateCalendar:
  [ .movie.name, (.time|calendarDate),"False",(.time|calendarTime),calendarEndTime,(.screenId|getScreen),"","False" ]
  ;
