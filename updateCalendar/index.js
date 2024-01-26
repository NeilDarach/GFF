const fs = require('fs').promises;
const path = require('path');
const process = require('process');
const {authenticate} = require('@google-cloud/local-auth');
const {google} = require('googleapis');

const SCOPES = [ 'https://www.googleapis.com/auth/calendar.readonly',
                 'https://www.googleapis.com/auth/calendar.events'
               ];

const TOKEN_PATH = path.join(process.cwd(),'token.json');
const CREDENTIALS_PATH = path.join(process.cwd(), 'credentials.json');

async function loadSavedCredentialsIfExist() {
  try {
    const content = await fs.readFile(TOKEN_PATH);
    const credentials = JSON.parse(content);
    return google.auth.fromJSON(credentials);
  } catch (err) {
    return null;
  }
}

async function saveCredentials(client) {
  const content = await fs.readFile(CREDENTIALS_PATH);
  const keys = JSON.parse(content);
  const key = keys.installed || keys.web;
  const payload = JSON.stringify({
    type: 'authorized_user',
    client_id: key.client_id,
    client_secret: key.client_secret,
    refresh_token: client.credentials.refresh_token,
  });
  await fs.writeFile(TOKEN_PATH, payload);
}

async function authorize() {
  let client = await loadSavedCredentialsIfExist();
  if (client) { return client; }
  client = await authenticate({
    scopes: SCOPES,
    keyfilePath: CREDENTIALS_PATH,
  })
  if (client.credentials) {
    await saveCredentials(client);
  }
  return client;
}

async function listEvents(auth) {
  const calendar = google.calendar({version: 'v3', auth});
  const res = await calendar.events.list({
    calendarId: 'c12717e59b8cbf4e58b2eb5b0fe0e8aa823cf71943cab642507715cd86db80f8@group.calendar.google.com',
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
  //events.map((event,i) => { 
    //const start = event.start.dateTime || event.start.date;
    //console.log(`${event.id}  ${start} - ${event.summary}`);
  //});
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

async function changeColor(auth,eventId,colorId) {
  console.log(`Changing color of ${eventId}`);
  const calendar = google.calendar({version: 'v3', auth});
  const res = await calendar.events.patch({
    auth: auth,
    calendarId: 'c12717e59b8cbf4e58b2eb5b0fe0e8aa823cf71943cab642507715cd86db80f8@group.calendar.google.com',
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

async function recolor(auth) {
  const events = await listEvents(auth);
  console.log(`Got ${events.length} events`);
  for (event of events) {
    switch(event.location) {
      case "GFT 1": colorId = 1; break;
      case "GFT 2": colorId = 2; break;
      case "GFT 3": colorId = 3; break;
      case "Cineworld Screen 1": colorId = 4; break;
      case "Cineworld Screen 2": colorId = 5; break;
      case "Cottiers": colorId = 6; break;
      case "Barras Art & Design (BAoD)": colorId = 7; break;
      case "CCA Cinema": colorId = 8; break;
      case "Cineworld 2": colorId = 9; break;
      default: colorID = 10;
      }
    console.log(`Changing ${events.summary} in ${events.location} to ${colorId}`);
    changeColor(auth,event.id,colorId); 
    await sleep(500);
  }
}


//authorize().then(listCalendars).catch(console.error)
//authorize().then(listEvents).catch(console.error)
//authorize().then((auth) => { changeColor(auth,'_8d9lcgrfdpr6asjkcgqj0cj56kp68e9mcpgjgob16cpj6dr36lj3aohjcgqm6d346osg') }).catch(console.error)
authorize().then((auth) => { recolor(auth) }).catch(console.error)
