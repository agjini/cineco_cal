use chrono::{DateTime, Local, TimeZone, Utc};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use crate::calendar::config::Config;
use crate::calendar::Movie;
use crate::error::Error;
use crate::error::Error::Unreachable;

pub async fn load_movies(location: &str) -> Result<Vec<Movie>, Error> {
    let config = Config::load();
    let html = parse_cinegestion(config).await?;

    let fragment = Html::parse_fragment(&html);
    let ul_selector = Selector::parse(r#"tr[data-type="show"]"#).unwrap();
    let td_selector = Selector::parse(r#"td"#).unwrap();
    let movies: Vec<_> = fragment.select(&ul_selector)
        .filter_map(|show| parse_movie(location, show, &td_selector))
        .collect();

    Ok(movies)
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
        let projector = tds[5].inner_html().trim().to_string();
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

fn parse_date(date: &str) -> Option<DateTime<Utc>> {
    let date_pattern: Regex = Regex::new(r"(?<day>\w+) (?<day_of_month>\d{2}) (?<month>\w+) (?<year>\d{4}) (?<hour>\d{2}):(?<min>\d{2})").unwrap();
    date_pattern.captures(date)
        .and_then(|re| Local.with_ymd_and_hms(
            re["year"].parse::<i32>().unwrap(),
            parse_month(&re["month"]),
            re["day_of_month"].parse::<u32>().unwrap(),
            re["hour"].parse::<u32>().unwrap(),
            re["min"].parse::<u32>().unwrap(),
            0,
        ).single())
        .map(|dt| dt.with_timezone(&Utc))
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

fn map_reqwest_error(e: reqwest::Error) -> Error {
    tracing::error!("Error while accessing cinegestion : {}", e);
    Unreachable("Error while accessing cinegestion".to_string())
}

async fn parse_cinegestion(config: Config) -> Result<String, Error> {
    let params = [("login", config.cinegestion_login), ("password", config.cinegestion_password)];
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .map_err(map_reqwest_error)?;

    let _ = client.post("https://cineco.cinegestion.fr/login")
        .form(&params)
        .send()
        .await
        .map_err(map_reqwest_error)?;

    let resp = client.get("https://cineco.cinegestion.fr/admin?all=24")
        .send()
        .await
        .map_err(map_reqwest_error)?;

    let html = resp.text().await
        .map_err(map_reqwest_error)?;
    Ok(html)
}

fn parse_firstname(assigned: &str) -> Option<String> {
    Regex::new(r"^(?<name>\w+)(\s(.*))?$").unwrap()
        .captures(assigned)
        .map(|c| c["name"].to_string())
}