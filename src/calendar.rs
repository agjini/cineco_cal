use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, Utc};
use ics::{escape_text, Event, ICalendar};
use ics::properties::{Attendee, Categories, Description, DtEnd, DtStart, Status, Summary};
use uuid::{Uuid, uuid};

use crate::calendar::cinegestion::load_movies;
use crate::error::Error;

mod cinegestion;
mod config;

const CINECO_UUID: Uuid = uuid!("4f345610-24a1-4c21-84cf-7f3efdf964d0");

#[derive(Debug)]
pub struct Movie {
    id: u32,
    title: String,
    date: DateTime<Utc>,
    projector: String,
    assigned_to: Vec<String>,
}

pub struct CinecoCalendar(ICalendar<'static>);

impl IntoResponse for CinecoCalendar {
    fn into_response(self) -> Response {
        let mut response = self.0.to_string().into_response();
        response.headers_mut().insert("Content-Type", "text/calendar".parse().unwrap());
        response
    }
}

pub async fn generate_calendar(Path((location, me)): Path<(String, String)>) -> Result<CinecoCalendar, Error> {
    let movies = load_movies(&location).await?;
    tracing::debug!("Found {:?} movie(s)", movies.len());

    let mut calendar = ICalendar::new("2.0", "-//xyz Corp//NONSGML PDA Calendar Version 1.0//EN");
    for movie in movies {
        let event = map_to_event(&me, movie);
        calendar.add_event(event);
    }

    Ok(CinecoCalendar(calendar))
}

fn map_to_event<'a>(me: &str, movie: Movie) -> Event<'a> {
    let uid = Uuid::new_v5(&CINECO_UUID, movie.id.to_string().as_bytes());
    let mut event = Event::new(uid.to_string(), format_date(&movie.date));
    for assigned in movie.assigned_to.clone() {
        if assigned.eq(me) {
            event.push(Status::confirmed());
        }
        event.push(Attendee::new(assigned));
    }
    event.push(DtStart::new(format_date(&movie.date)));
    let end = movie.date + Duration::hours(2);
    event.push(DtEnd::new(format_date(&end)));
    event.push(Categories::new("PROJECTION"));
    event.push(Categories::new("CINEMA"));
    event.push(Summary::new(movie.title.clone()));
    event.push(Description::new(escape_text(
        format!("Numéro de séance: {}\n\
            Projection du film '{}'\n\
            Projectioniste(s): {}\n\
            Projo: {}",
                movie.id, &movie.title, movie.assigned_to.join(", "), movie.projector)
    )));
    event
}

fn format_date(date: &DateTime<Utc>) -> String {
    date.format("%Y%m%dT%H%M%SZ").to_string()
}