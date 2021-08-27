use chrono::{DateTime, Local};
use clap::{AppSettings, Clap};
use colored::*;
use serde::Deserialize;
use std::iter::Iterator;
use textwrap::{fill, indent};

#[derive(Deserialize, Debug)]
struct Hour {
    from: String,
    to: String,
    r#type: String,
}

#[derive(Deserialize, Debug)]
struct Hours {
    opening: Vec<Hour>,
    mealtime: Vec<Hour>,
}

#[derive(Deserialize, Debug)]
struct Location {
    id: u32,
    label: String,
}

#[derive(Deserialize, Debug)]
struct Meal {
    id: u32,
    mealtypes: Vec<MealType>,
    label: String,
    description: Vec<String>,
    position: u32,
    prices: Prices,
    allergenes: Option<Vec<Allergene>>,
    origins: Option<Vec<Origin>>,
}

#[derive(Deserialize, Debug)]
struct MealType {
    mealtype_id: u32,
    label: String,
}

#[derive(Deserialize, Debug)]
struct Allergene {
    allergene_id: u32,
    label: String,
}

#[derive(Deserialize, Debug)]
struct Origin {
    origin_id: u32,
    label: String,
}

#[derive(Deserialize, Debug)]
struct Prices {
    student: Option<String>,
    staff: Option<String>,
    r#extern: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Mensa {
    id: u32,
    daytime: String,
    mensa: String,
    hours: Hours,
    location: Location,
    meals: Vec<Meal>,
}

#[derive(Clap)]
#[clap(version = "0.1", author = "Nicolas Ochsner <nicolasochsner@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(default_value = "poly")]
    input: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let baseurl = "https://www.webservices.ethz.ch/gastro/v1/RVRI/Q1E1";
    let lang = "en";
    let localtime: DateTime<Local> = Local::now();

    let meals_url = format!(
        "{baseurl}/meals/{lang}/{date}/{time}",
        baseurl = baseurl,
        lang = lang,
        date = format!("{}", localtime.format("%F")),
        time = "lunch",
    );

    let meals_response = reqwest::get(&meals_url).await?;
    let mensa_meals: Vec<Mensa> = meals_response.json().await?;

    match mensa_meals
        .iter()
        .find(|&m| m.mensa.to_lowercase().contains(&opts.input.to_lowercase()))
    {
        Some(mensa) => {
            for meal in &mensa.meals {
                println!("{}", meal.label.bold());
                println!("{}", meal.description[0]);
                for i in 1..meal.description.len() {
                    println!(
                        "{}",
                        indent(fill(meal.description[i].as_str(), 40).as_str(), "    ")
                    );
                }
                println!("");
            }
        }
        None => {
            println!("Could not find mensa, it might be closed today.");
        }
    };

    Ok(())
}
