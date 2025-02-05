use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Month {
    pub year: String,
    pub month: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Day {
    day: String,
    pages: Vec<Page>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    page: String,
    url: String
}
