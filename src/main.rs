mod cli;

use crate::cli::Opts;
use chrono::{DateTime, Local};
use clap::Parser;
use colored::*;
use serde::Deserialize;
use std::fmt;
use std::iter::Iterator;
use textwrap::{fill, indent};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Hour {
    from: String,
    to: String,
    r#type: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Hours {
    opening: Vec<Hour>,
    mealtime: Vec<Hour>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Location {
    id: u32,
    label: String,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct MealType {
    mealtype_id: u32,
    label: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Allergene {
    allergene_id: u32,
    label: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Origin {
    origin_id: u32,
    label: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Prices {
    student: Option<String>,
    staff: Option<String>,
    r#extern: Option<String>,
}

impl fmt::Display for Prices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep = match (
            self.student.as_ref(),
            self.staff.as_ref(),
            self.r#extern.as_ref(),
        ) {
            (Some(a), Some(b), Some(c)) => format!("{}/{}/{}", a, b, c),
            (Some(a), None, None) => a.clone(),
            (None, Some(b), None) => b.clone(),
            (None, None, Some(c)) => c.clone(),
            (Some(a), Some(b), None) => format!("{}/{}", a, b),
            (None, Some(b), Some(c)) => format!("{}/{}", b, c),
            (Some(a), None, Some(c)) => format!("{}/{}", a, c),
            (None, None, None) => "".to_string(),
        };
        write!(f, "{}", rep)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Mensa {
    id: u32,
    daytime: String,
    mensa: String,
    hours: Hours,
    location: Location,
    meals: Vec<Meal>,
}

fn show_available_mensas(mensa_meals: Vec<Mensa>) {
    for mensa in mensa_meals {
        println!("{}", mensa.mensa)
    }
}

fn show_meals_for_mensa(mensa_meals: Vec<Mensa>, mensa_name: &str, show_prices: bool) {
    match mensa_meals
        .iter()
        .find(|&m| m.mensa.to_lowercase().contains(&mensa_name.to_lowercase()))
    {
        Some(mensa) => {
            for meal in &mensa.meals {
                println!("{}", meal.label.bold());
                println!("{}", meal.description[0]);
                for i in 1..meal.description.len() {
                    if !meal.description[i].is_empty() {
                        println!(
                            "{}",
                            indent(fill(meal.description[i].as_str(), 40).as_str(), "    ")
                        );
                    }
                }
                if show_prices {
                    println!("\t\t\t\t{}", meal.prices);
                }
                println!();
            }
        }
        None => {
            println!("Could not find mensa, it might be closed today.");
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let baseurl = "https://www.webservices.ethz.ch/gastro/v1/RVRI/Q1E1";
    let localtime: DateTime<Local> = Local::now();

    let meals_url = format!(
        "{baseurl}/meals/{lang}/{date}/{time}",
        baseurl = baseurl,
        lang = opts.lang,
        date = format!("{}", localtime.format("%F")),
        time = "lunch",
    );

    let meals_response = reqwest::get(&meals_url).await?;
    let mensa_meals: Vec<Mensa> = meals_response.json().await?;

    if opts.list {
        show_available_mensas(mensa_meals);
    } else {
        show_meals_for_mensa(mensa_meals, &opts.mensa, opts.prices);
    }
    Ok(())
}
