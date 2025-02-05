use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Month {
    pub year: String,
    pub month: String,
    pub url: String,
    pub days: Vec<Day>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Day {
    pub day: String,
    pub root_url: String
}
