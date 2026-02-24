calendar-access gets setColours to update the google calendar displayed colour based on screen
gffd
FestivalEvent::fetch_from_gft takes a movie_id and returns a vec of Event instances corresponding to screenings.  events are cached.
fetch_image takes a key from event and retrieves an image from the server, returning for cache if available.
args are
    auth:   auth.json for google
    main:   calendar id for the complete calendar
    filter: calendar id for the filtered calendar showing only attended screenings

Commands are 
    Serve   start a web server to respond to request.  Not yet implemented.

    Ids     dump the contents of the film-to-id map
    FetchScreenings pull screening details for know ids from the GFT site
    ShowConfig  dump the merged config
    List

