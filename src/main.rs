mod cli;

use crate::cli::{Opts, Commands};
use chrono::prelude::*;
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use clap::Parser;
use colored::*;
use derive_more::{Deref, From, Into};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;
use textwrap::{fill, indent};
use std::fs;
use std::path::Path;


#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Facility {
    facility_id: usize,
    facility_name: String,
    facility_url: Option<String>,

    building: String,
    floor: String,

    address_line_2: String,
    address_line_3: String,
    phone: Option<String>,

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
enum MensarError<'a> {
    FacilityNotFound(&'a str),
    NoDailyMeals(&'a str),
    DefaultsError(&'static str),
}

impl<'a> fmt::Display for MensarError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            MensarError::FacilityNotFound(facility) => {
                format!("could not find facility `{facility}`")
            }
            MensarError::NoDailyMeals(facility) => {
                format!("no daily meals for `{facility}`")
            }
            MensarError::DefaultsError(msg) => {
                format!("unable to load defaults: `{msg}`")
            }
        };
        write!(f, "mensar: {msg}")?;
        Ok(())
    }
}

impl<'a> std::error::Error for MensarError<'a> {}

fn exit(error: MensarError) -> ! {
    println!("{}", error);
    std::process::exit(1);
}

#[derive(Debug, Deserialize, Serialize)]
struct Defaults {
    #[serde(default = "Defaults::default_mensa")]
    mensa: String,
    #[serde(default = "Defaults::default_lang")]
    lang: String,
}

impl Defaults {
    fn default_mensa() -> String {
        "polyterrasse".to_string()
    }

    fn default_lang() -> String {
        "en".to_string()
    }

    fn load() -> Result<Self, MensarError<'static>> {
        let home = std::env::var("HOME").unwrap();
        let defaults_path = Path::new(&home).join(".local/share/mensar/defaults.toml");
        let content = fs::read_to_string(&defaults_path).unwrap();
        toml::from_str(&content).map_err(|_e| MensarError::DefaultsError("unable to parse defaults"))
    }

    fn store(&self) -> Result<(), MensarError> {
        let home = std::env::var("HOME").unwrap();
        let defaults_path = Path::new(&home).join(".local/share/mensar/defaults.toml");
        let content = toml::to_string(self).map_err(|_e| MensarError::DefaultsError("unable to write defaults"))?;
        fs::write(defaults_path, content.as_str()).map_err(|_e| MensarError::DefaultsError("unable to write defaults"))
    }

    fn set_mensa(&mut self, value: String) {
        self.mensa = value;
    }

    fn set_lang(&mut self, value: String) {
        self.lang = value;
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    match opts.command {
        Some(Commands::SetMensa { name }) => {
            let mut defaults = Defaults::load().unwrap();
            defaults.set_mensa(name.clone());
            defaults.store().unwrap();
            println!("mensar: stored default mensa `{name}`");
            return Ok(());
        },
        Some(Commands::SetLanguage { name }) => {
            let mut defaults = Defaults::load().unwrap();
            defaults.set_lang(name.clone());
            defaults.store().unwrap();
            println!("mensar: stored default mensa `{name}`");
            return Ok(());
        },
        None => {}
    }

    let defaults = Defaults::load().unwrap();

    let mensa = opts.mensa.unwrap_or(defaults.mensa);
    let lang = opts.lang.unwrap_or(defaults.lang);

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
            .contains(&mensa.to_lowercase())
    }) {
        Some(f) => f,
        None => exit(MensarError::FacilityNotFound(&mensa)),
    };

    let date = localtime.format("%F");
    let meals_url = format!(
        "{baseurl}/weeklyrotas?client-id=ethz-wcms&lang={lang}&rs-first=0&rs-size=50&valid-after={date}");

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
        .unwrap_or_else(|| exit(MensarError::NoDailyMeals(&mensa)))
        .day_of_week_array
        .iter()
        .find(|d| d.day_of_week_code == weekday);

    let opening_hours = &daily_meals
        .unwrap_or_else(|| exit(MensarError::NoDailyMeals(&mensa)))
        .opening_hour_array;
    let opening_hour = &opening_hours
        .as_ref()
        .unwrap_or_else(|| exit(MensarError::NoDailyMeals(&mensa)))[0];

    let meal_times = &opening_hour
        .meal_time_array
        .as_ref()
        .unwrap_or_else(|| exit(MensarError::NoDailyMeals(&mensa)));
    let meal_time = &meal_times[0];

    let meals = meal_time
        .line_array
        .as_ref()
        .unwrap_or_else(|| exit(MensarError::NoDailyMeals(&mensa)));

    for meal in meals {
        println!("{}", meal.name.bold());
        if let Some(meal_details) = &meal.meal {
            println!("{}", indent(meal_details.name.as_str(), "    "));
            println!(
                "{}",
                indent(fill(meal_details.description.as_str(), 40).as_str(), "    ")
            );
            if opts.prices {
                let prices = meal_details
                    .meal_price_array
                    .iter()
                    .map(|mp| format!("{}", mp.price))
                    .collect::<Vec<String>>()
                    .join(" / ");
                println!("{}", indent(prices.as_str(), "\t\t\t\t"));
            }
        }
    }

    Ok(())
}
