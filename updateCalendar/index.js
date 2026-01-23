const fs = require('fs').promises;
const path = require('path');
const process = require('process');
const {google} = require('googleapis');
const serviceAccount = new google.auth.GoogleAuth ({
    keyFile: "../google-auth.json",
    scopes: [ 'https://www.googleapis.com/auth/calendar',
              'https://www.googleapis.com/auth/calendar.events'
            ], });

const TEST_CALENDAR = 'cfc6770c1350359a9a5012005a502636f82734ab014cc402882adde95ceb99c5@group.calendar.google.com';
const LIVE_CALENDAR = 'c12717e59b8cbf4e58b2eb5b0fe0e8aa823cf71943cab642507715cd86db80f8@group.calendar.google.com';
google.options({ auth: serviceAccount });


async function listEvents() {
  const calendar = google.calendar('v3');
  const res = await calendar.events.list({
    calendarId: LIVE_CALENDAR,
    timeMin: new Date().toISOString(),
    maxResults: 1000,
    singleEvents: true,
    orderBy: 'startTime',
  });
  const events = res.data.items;
  if (!events || events.length == 0) {
    console.log("No upcoming events found");
    return [];
  }
  events.map((event,i) => { 
    const start = event.start.dateTime || event.start.date;
    //console.log(`${start} - ${event.summary}`);
  });
  return events
}

async function listCalendars(auth) {
  const calendar = google.calendar({version: 'v3', auth});
  const res = await calendar.calendarList.list();
  const calendars = res.data.items;
  if (!calendars || calendars.length == 0) {
    console.log("No calendars found");
    return;
  }
  calendars.map((cal,i) => { 
    console.log(`${cal.summary} - ${cal.id}`);
  });
}

async function fakeChangeColor(auth,eventId,colorId) {
  console.log(`Changing color of ${eventId}`);
  }

async function changeColor(eventId,colorId) {
  console.log(`Changing color of ${eventId}`);
  const calendar = google.calendar('v3');
  const res = await calendar.events.patch({
    calendarId: LIVE_CALENDAR,
    eventId: eventId,
    resource: { colorId: colorId }
  });
  console.log(res);
}

// 1 dusky purple
// 2 green
// 3 purple
// 4 dusky pink
// 5 yellow
// 6 red
// 7 blue
// 8 grey

function sleep(ms) {
  return new Promise((resolve) => { setTimeout(resolve,ms); });
}

async function recolor() {
  const events = await listEvents();
  console.log(`Got ${events.length} events`);
  for (event of events) {
    switch(event.location) {
      case "GFT 1": colorId = 1; break;
      case "GFT 2": colorId = 2; break;
      case "GFT 3": colorId = 3; break;
      case "Odeon 10": colorId = 4; break;
      case "Odeon 11": colorId = 5; break;
      case "Odeon 12": colorId = 6; break;
      case "Pyramid": colorId = 7; break;
      case "Special": colorId = 8; break;
      case "PWYC": colorId = 9; break;
      default: colorID = 10;
      }
        if (event.colorId != colorId) {
    console.log(`Changing ${event.summary} in ${event.location} to ${colorId}`);
    changeColor(event.id,colorId); 
    await sleep(300);
        }
  }
}

async function clear() {
  const events = await listEvents();
  const calendar = google.calendar('v3');
  console.log(`Got ${events.length} events`);
  for (event of events) {
    const res = await calendar.events.delete({
    calendarId: LIVE_CALENDAR,
    eventId: event.id,
  });
  console.log(res);
    await sleep(500);
  }
}

//authorize().then(listCalendars).catch(console.error)
//authorize().then(listEvents).catch(console.error)
//authorize().then((auth) => { changeColor(auth,'_8d9lcgrfdpr6asjkcgqj0cj56kp68e9mcpgjgob16cpj6dr36lj3aohjcgqm6d346osg') }).catch(console.error)
//authorize().then((auth) => { recolor(auth) }).catch(console.error)
(async () => { await recolor(); })();
