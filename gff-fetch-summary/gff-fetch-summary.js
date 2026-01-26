#!/usr/bin/env node

const fs = require('fs').promises;
const path = require('path');
const process = require('process');
const {google} = require('googleapis');
const serviceAccount = new google.auth.GoogleAuth ({
    keyFile: process.env.GFF_AUTH,
    scopes: [ 'https://www.googleapis.com/auth/calendar',
              'https://www.googleapis.com/auth/calendar.events'
            ], });

const LIVE_CALENDAR = process.env.GFF_FILTER_ID;
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
    var byDate = {};
    var screens = { "GFT1": "GFT 1" };
    var days = [ "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday" ];
  events.map((event,i) => { 
    // 2026-02-25T19:00:00Z - Opening Gala: Everybody to Kenmure Street - GFT1 - M N V

        //console.log(`${JSON.stringify(event)}`);
    const start_dt = new Date(event.start.dateTime);
    const end_dt = new Date(event.end.dateTime);
    const duration = (end_dt - start_dt)/ 60000;
    const date = start_dt.getFullYear() + "-" + ("0"+(start_dt.getMonth()+1)).slice(-2) + "-" + ("0" +(start_dt.getDate())).slice(-2);
    const start = ("0" + start_dt.getHours()).slice(-2) + ":" + ("0"+start_dt.getMinutes()).slice(-2);
    const day = days[start_dt.getDay()];
    pos = event.summary.lastIndexOf(" - ");
    const attendees = event.summary.slice(pos+3).split(" ");
    pos2 = event.summary.lastIndexOf(" - ",pos-1);
    const screen = event.extendedProperties.shared.screen;
        strand = event.extendedProperties.shared.strand;
        color = event.extendedProperties.shared.color;
    const title = event.summary.substring(0,pos2);
    if (byDate[date] == undefined) { byDate[date] = {} ; }
    if (byDate[date][screen] == undefined) { byDate[date][screen] = [] ; }
    byDate[date][screen].push({ title: title, start: start, duration: duration , day: day, attendees: attendees, color: color, strand: strand});
    //console.log(`${date} - ${start} - ${duration} - ${title} - ${screen} - ${attendees}`);
        //console.log(`${JSON.stringify(event)}`);
  });
    console.log(JSON.stringify(byDate));
  return events
}

(async () => { await listEvents(); })();
