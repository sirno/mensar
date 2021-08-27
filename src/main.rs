use chrono::{DateTime, Local};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let localtime: DateTime<Local> = Local::now();
    let mensa_url = format!(
        "https://www.webservices.ethz.ch/gastro/v1/RVRI/Q1E1/meals/{lang}/{date}/{time}",
        lang = "en",
        date = format!("{}", localtime.format("%F")),
        time = "lunch",
    );
    let response = reqwest::get(&mensa_url).await?;
    let mensas: Vec<Mensa> = response.json().await?;

    let pt = mensas.iter().find(|&m| m.id == 12).unwrap();

    for meal in &pt.meals {
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
    Ok(())
}
