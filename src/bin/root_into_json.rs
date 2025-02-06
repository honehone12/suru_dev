use std::env;
use anyhow::bail;
use tokio::fs;
use scraper::{Html, Selector};
use suru_dev::MonthlyRoot;

const SOURCE: &'static str = "html/root.html";
const DEST: &'static str = "json/root.json";

async fn scrape_root(document: String) -> anyhow::Result<Vec<MonthlyRoot>> {
    let doc = Html::parse_document(&document);
    
    let table_tag = match Selector::parse("table") {
        Ok(t) => t,
        Err(e) => bail!("{e}")
    };
    let row_tag = match Selector::parse("tr") {
        Ok(t) => t,
        Err(e) => bail!("{e}")
    };
    let data_tag = match Selector::parse("td") {
        Ok(t) => t,
        Err(e) => bail!("{e}")
    };
    let a_tag = match Selector::parse("a") {
        Ok(t) => t,
        Err(e) => bail!("{e}")
    };

    let root = env::var("URL_ROOT")?;
    let mut list = vec![];

    for table in doc.select(&table_tag) {
        for row in table.select(&row_tag) {
            for data in row.select(&data_tag) {
                for a in data.select(&a_tag) {
                    let Some(href) =  a.attr("href") else {
                        continue;
                    };

                    let mut url = format!("{root}{href}");
                    if url.ends_with("/") {
                        url.pop();
                    }

                    let month = MonthlyRoot::from_url(url)?;
                    list.push(month);
                }
            }
        }
    }
    
    Ok(list)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;

    let document = fs::read_to_string(SOURCE).await?;
    let list = scrape_root(document).await?;    
    let json = serde_json::to_string_pretty(&list)?;
    fs::write(DEST, json).await?;

    Ok(())
}
