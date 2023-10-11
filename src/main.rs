mod cli;

use crate::cli::Opts;
use chrono::prelude::*;
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use clap::Parser;
use colored::*;
use derive_more::{Deref, From, Into};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;
use textwrap::{fill, indent};

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Facility {
    facility_id: usize,
    facility_name: String,
    facility_url: String,

    building: String,
    floor: String,

    address_line_2: String,
    address_line_3: String,
    phone: String,

    caterer_name: Option<String>,
    caterer_url: Option<String>,

    publication_type_code: usize,
    publication_type_desc: String,
    publication_type_desc_short: String,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct WeeklyRotas {
    weekly_rota_id: usize,
    facility_id: usize,
    valid_from: NaiveDate,
    valid_to: Option<NaiveDate>,
    day_of_week_array: Vec<DayOfWeek>,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct DayOfWeek {
    day_of_week_code: u32,
    day_of_week_desc: String,
    day_of_week_desc_short: String,
    opening_hour_array: Option<Vec<OpeningHour>>,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Times {
    time_from: NaiveTime,
    time_to: NaiveTime,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct OpeningHour {
    #[serde(flatten)]
    times: Times,
    meal_time_array: Option<Vec<MealTime>>,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct MealTime {
    name: String,
    #[serde(flatten)]
    times: Times,
    line_array: Option<Vec<LineElement>>,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct LineElement {
    name: String,
    meal: Option<Meal>,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Meal {
    name: String,
    description: String,
    meal_price_array: Vec<MealPrice>,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct MealPrice {
    price: f64,
    customer_group_code: usize,
    customer_group_position: usize,
    customer_group_desc: String,
    customer_group_desc_short: String,
}

#[derive(Clone, Debug, Deref, Into, From)]
struct Facilities(Vec<Facility>);

impl fmt::Display for Facilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for facility in &self.0 {
            if facility.publication_type_code == 1 {
                writeln!(f, "{}", facility.facility_name)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum MensarError {
    FacilityNotFound(String),
}

impl fmt::Display for MensarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            MensarError::FacilityNotFound(facility) => {
                format!("could not find facility `{facility}`")
            }
        };
        write!(f, "mensar: {msg}")?;
        Ok(())
    }
}

impl std::error::Error for MensarError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let baseurl = "https://idapps.ethz.ch/cookpit-pub-services/v1";
    let localtime: DateTime<Local> = Local::now();

    let facilities_url =
        format!("{baseurl}/facilities?client-id=ethz-wcms&lang=en&rs-first=0&rs-size=50");
    let facilities_response = reqwest::get(&facilities_url).await?;
    let facilities_json: HashMap<String, Vec<Facility>> = facilities_response.json().await?;
    let facilities: Facilities = Facilities::from(facilities_json["facility-array"].clone());

    if opts.list {
        print!("{}", &facilities);
        return Ok(());
    }

    let facility = match facilities.iter().find(|&f| {
        f.facility_name
            .to_lowercase()
            .contains(&opts.mensa.to_lowercase())
    }) {
        Some(f) => f,
        None => {
            println!("{}", MensarError::FacilityNotFound(opts.mensa));
            std::process::exit(1);
        }
    };

    // dbg!(&facility);

    let date = localtime.format("%F");
    let meals_url = format!(
        "{baseurl}/weeklyrotas?client-id=ethz-wcms&lang={lang}&rs-first=0&rs-size=50&valid-after={date}", lang = opts.lang);

    let meals_response = reqwest::get(&meals_url).await?;
    let meals_json: HashMap<String, Vec<WeeklyRotas>> = meals_response.json().await?;
    let meals = &meals_json["weekly-rota-array"];

    let naive_date = localtime.date_naive();
    let facility_meals = meals.iter().find(|m| {
        m.facility_id == facility.facility_id
            && m.valid_from <= naive_date
            && (m.valid_to.is_none() || m.valid_to.unwrap() >= naive_date)
    });
    let weekday = localtime.weekday().number_from_monday();
    let daily_meals = facility_meals
        .unwrap()
        .day_of_week_array
        .iter()
        .find(|d| d.day_of_week_code == weekday);
    let opening_hours = daily_meals.unwrap().opening_hour_array.clone();
    let opening_hour = &opening_hours.unwrap()[0];

    dbg!(opening_hour);

    Ok(())
}
