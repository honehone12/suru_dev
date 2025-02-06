use anyhow::bail;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyRoot {
    pub year: u32,
    pub month: u32,
    pub url: String,
    pub products: Vec<DailyRoot>
}

impl MonthlyRoot {
    pub fn from_url(url: String) -> anyhow::Result<MonthlyRoot> {
        let Some((_, ym)) = url.rsplit_once("/") else {
            bail!("could not split with '/': {url}");
        };
    
        let Some((y, m)) = ym.split_at_checked(4) else {
            bail!("could not split at 4: {url}");
        };
    
        if y.len() != 4 {
            bail!("unexpected year length: {url}");
        }
        if m.len() != 2 {
            bail!("unexpected month length: {url}");
        }
    
        Ok(Self{
            year: y.parse()?,
            month: m.parse()?,
            url,
            products: vec![]
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyRoot {
    pub day: u32,
    pub url: String
}

impl DailyRoot {
    pub fn from_url(url: String) -> anyhow::Result<DailyRoot> {
        let Some((_, ymd_p)) = url.rsplit_once("/") else {
            bail!("failed to split with '/': {url}");
        };
    
        let Some(d) = ymd_p.get(6..=7) else {
            bail!("failed to get range 6..=7: {url}");
        };

        Ok(Self{
            day: d.parse()?,
            url
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyProducts {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub products: Vec<ProductDescription>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDescription {
    pub description: String,
    pub url: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub price: u64,
    pub original_price: u64    
}
