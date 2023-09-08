#[macro_use]
extern crate rocket;

use chrono::{DateTime, Duration, Local, TimeZone};
use ics::{escape_text, Event, ICalendar};
use ics::properties::{Attendee, Categories, Description, DtEnd, DtStart, Status, Summary};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![generate_calendar])
}

#[derive(Debug)]
struct Movie {
    id: u32,
    title: String,
    date: DateTime<Local>,
    projector: Option<String>,
    assigned_to: Vec<String>,
}

struct Config {
    cinegestion_login: String,
    cinegestion_password: String,
}

#[get("/<location>/<me>")]
async fn generate_calendar(location: &str, me: &str) -> Result<String, String> {
    let config = load_config();
    let html = parse_cinegestion(config).await?;

    let fragment = Html::parse_fragment(&html);
    let ul_selector = Selector::parse(r#"tr[data-type="show"]"#).unwrap();
    let td_selector = Selector::parse(r#"td"#).unwrap();

    let movies: Vec<_> = fragment.select(&ul_selector)
        .filter_map(|show| parse_movie(location, show, &td_selector))
        .collect();
    println!("Found {:?} movie(s)", movies.len());

    // create new iCalendar object
    let mut calendar = ICalendar::new("2.0", "-//xyz Corp//NONSGML PDA Calendar Version 1.0//EN");

    for movie in movies {
        // create event which contains the information regarding the conference
        let mut event = Event::new(movie.id.to_string(), movie.date.to_rfc3339());
        for assigned in movie.assigned_to.clone() {
            if assigned.eq(me) {
                event.push(Status::confirmed());
            }
            event.push(Attendee::new(assigned));
        }
        event.push(DtStart::new(movie.date.to_rfc3339()));
        let end = movie.date + Duration::hours(2);
        event.push(DtEnd::new(end.to_rfc3339()));
        event.push(Categories::new("PROJECTION"));
        event.push(Categories::new("CINEMA"));
        event.push(Summary::new(movie.title.clone()));
        event.push(Description::new(escape_text(
            format!("Numéro de séance: {}\nProjection du film '{}'\nProjectioniste(s): {}\nProjo: {}", movie.id, &movie.title, movie.assigned_to.join(", "), movie.projector.unwrap_or("N/A".to_string()))
        )));
        calendar.add_event(event);
    }

    Ok(calendar.to_string())
}

fn load_config() -> Config {
    Config {
        cinegestion_login: std::env::var("CINEGESTION_LOGIN").expect("Miss CINEGESTION_LOGIN"),
        cinegestion_password: std::env::var("CINEGESTION_PASSWORD").expect("Miss CINEGESTION_PASSWORD"),
    }
}

fn parse_movie(location: &str, show: ElementRef, td_selector: &Selector) -> Option<Movie> {
    let tds: Vec<_> = show.select(td_selector)
        .collect();
    let voluntary = tds[1].inner_html();
    let movie_location = tds[2].inner_html();
    if tds.len() < 7 || !voluntary.eq("Benevoles") || !movie_location.eq(location) {
        None
    } else {
        let date = parse_date(&tds[3].inner_html())?;
        let title = parse_title(tds[4].inner_html());
        let projector = match tds[5].inner_html().trim() {
            "N/A" => None,
            s => Some(s.to_string()),
        };
        let assigned_to = match tds[6].value().attr("data-names") {
            None => vec![],
            Some(v) => v.split(", ")
                .filter_map(parse_firstname)
                .collect()
        };
        Some(Movie {
            id: tds[0].inner_html().parse::<u32>().unwrap(),
            title,
            date,
            projector,
            assigned_to,
        })
    }
}

fn parse_title(title_html: String) -> String {
    Regex::new(r"(?<title>.*)\s+<br>.*")
        .unwrap()
        .captures(&title_html)
        .map(|re| re["title"].trim().to_string())
        .unwrap_or(title_html)
}

fn parse_date(date: &str) -> Option<DateTime<Local>> {
    let date_pattern: Regex = Regex::new(r"(?<day>\w+) (?<day_of_month>\d{2}) (?<month>\w+) (?<year>\d{4}) (?<hour>\d{2}):(?<min>\d{2})").unwrap();
    date_pattern.captures(date)
        .map(|re| Local.with_ymd_and_hms(
            re["year"].parse::<i32>().unwrap(),
            parse_month(&re["month"]),
            re["day_of_month"].parse::<u32>().unwrap(),
            re["hour"].parse::<u32>().unwrap(),
            re["min"].parse::<u32>().unwrap(),
            0,
        ).single())?
}

fn parse_month(month: &str) -> u32 {
    match month {
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
        _ => 12
    }
}

async fn parse_cinegestion(config: Config) -> Result<String, String> {
    let params = [("login", config.cinegestion_login), ("password", config.cinegestion_password)];
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| e.to_string())?;

    let _ = client.post("https://cineco.cinegestion.fr/login")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let resp = client.get("https://cineco.cinegestion.fr/admin")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let html = resp.text().await.map_err(|e| e.to_string())?;
    Ok(html)
}

fn parse_firstname(assigned: &str) -> Option<String> {
    Regex::new(r"^(?<name>\w+)(\s(.*))?$").unwrap()
        .captures(assigned)
        .map(|c| c["name"].to_string())
}