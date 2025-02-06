use std::{time::{Duration, Instant}, env};
use anyhow::bail;
use tokio::{fs, time::sleep};
use suru_dev::{DailyRoot, MonthlyRoot};
use tracing::info;
use scraper::{Html, Selector};
use reqwest::Client as HttpClient;

const SOURCE: &'static str = "json/root.json";

async fn fill_root(
    client: &HttpClient, 
    a_tag: &Selector, 
    month: &mut MonthlyRoot,
    root: &str
) -> anyhow::Result<()> {
    info!("requesting {}/{}", month.year, month.month);

    let html = client.get(&month.url)
        .send().await?
        .text().await?;
    let document = Html::parse_document(&html);
    for a in document.select(&a_tag) {
        let Some(href) = a.attr("href") else {
            continue;
        };
        if !href.ends_with("_1") {
            continue;
        }

        let url = format!("{root}/{href}");
        let day = DailyRoot::from_url(url)?;
        if month.products.iter().find(|d| d.day == day.day).is_some() {
            continue;
        }
        month.products.push(day);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;

    let s = fs::read_to_string(SOURCE).await?;
    fs::write(format!("{SOURCE}.bu"), &s).await?;
    let mut list = serde_json::from_str::<Vec<MonthlyRoot>>(&s)?;
    let root = env::var("URL_ROOT")?;

    let a_tag = match Selector::parse("a") {
        Ok(t) => t,
        Err(e) => bail!("{e}")
    };

    let target_list = vec![];
    let client = HttpClient::new();
    let interval = Duration::from_millis(750);
    for month in list.iter_mut() {
        if !target_list.is_empty() && !target_list.contains(&month.year) {
            continue;
        }

        fill_root(&client, &a_tag, month, &root).await?;

        sleep(interval).await;
    }

    let json = serde_json::to_string_pretty(&list)?;
    fs::write(SOURCE, json).await?;

    info!("done in {}milsecs", start.elapsed().as_millis());
    Ok(())
}