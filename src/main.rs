#[macro_use]
extern crate rocket;

use std::fs;

use chrono::{DateTime, Local, Timelike, TimeZone};
use ics::{escape_text, Event, ICalendar};
use ics::properties::{Attendee, Categories, Description, DtEnd, DtStart, Status, Summary};
use regex::Regex;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![generate_calendar])
}

#[derive(Debug)]
struct Movie {
    id: u32,
    title: String,
    date: DateTime<Local>,
    has_short: bool,
    assigned_to: Vec<String>,
}

#[get("/")]
fn generate_calendar() -> Result<String, String> {
    let file_content = fs::read_to_string("input.txt")
        .map_err(|e| e.to_string())?;

    let re = Regex::new(r"(?<id>\d+)\s+Benevoles\s+Ste-Enimie\s+(?<day>\w+)\s+(?<day_of_month>\d\d)\s+(?<month>\w+)\s+(?<year>\d\d\d\d)\s+(?<hour>\d\d+):(?<min>\d\d+)\s+(?<title>.*)\n(court-métrage:\s(?<short_number>\d+))?\s+(?<projo>\S+)\s+(?<assigned_to>.*)Chèques\nLiquide").unwrap();
    let mut movies = vec![];
    for c in re.captures_iter(&file_content) {
        let date = Local.with_ymd_and_hms(
            c["year"].parse().unwrap(),
            match &c["month"] {
                "janvier" => 1,
                "février" => 2,
                "mars" => 3,
                "avril" => 4,
                "mai" => 5,
                "juin" => 6,
                "juillet" => 7,
                "août" => 8,
                "septembre" => 9,
                "octobre" => 10,
                "novembre" => 11,
                "décembre" => 12,
                _ => panic!("unknown month"),
            },
            c["day_of_month"].parse().unwrap(),
            c["hour"].parse().unwrap(),
            c["min"].parse().unwrap(),
            0,
        ).unwrap();
        movies.push(Movie {
            id: c["id"].parse().unwrap(),
            title: c["title"].to_string(),
            date,
            has_short: c.name("short_number").is_some(),
            assigned_to: c["assigned_to"].split(", ").map(|s| parse_firstname(s)).filter_map(|f| f).collect(),
        });
    }
    println!("Found {:?} movie(s)", movies.len());

    // create new iCalendar object
    let mut calendar = ICalendar::new("2.0", "-//xyz Corp//NONSGML PDA Calendar Version 1.0//EN");

    for movie in movies {
        // create event which contains the information regarding the conference
        let mut event = Event::new(movie.id.to_string(), movie.date.to_rfc3339());
        for assigned in movie.assigned_to.clone() {
            if assigned.eq("Augustin") {
                event.push(Status::confirmed());
            }
            event.push(Attendee::new(assigned));
        }
        event.push(DtStart::new(movie.date.to_rfc3339()));
        if let Some(end) = movie.date.with_hour(2) {
            event.push(DtEnd::new(end.to_rfc3339()));
        }
        event.push(Categories::new("PROJECTION"));
        event.push(Categories::new("CINEMA"));
        event.push(Summary::new(movie.title.clone()));
        event.push(Description::new(escape_text(
            format!("Numéro de séance: {}\nProjection du film '{}'\nProjectioniste(s): {}\nCourt métrage : {}", movie.id, &movie.title, movie.assigned_to.join(", "), if movie.has_short { "Oui" } else { "Non" })
        )));
        calendar.add_event(event);
    }

    Ok(calendar.to_string())
}

fn parse_firstname(assigned: &str) -> Option<String> {
    Regex::new(r"^(?<name>\w+)(\s(.*))?$").unwrap()
        .captures(assigned)
        .map(|c| c["name"].to_string())
}