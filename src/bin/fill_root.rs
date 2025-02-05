use std::{time::{Duration, Instant}, env};
use anyhow::bail;
use tokio::{fs, time::sleep};
use suru_dev::{Day, Month};
use tracing::info;
use scraper::{Html, Selector};
use reqwest::Client as HttpClient;

const SOURCE: &'static str = "json/root.json";

fn into_day(url: String) -> anyhow::Result<Day> {
    let Some((_, ymd_p)) = url.rsplit_once("/") else {
        bail!("failed to split with '/': {url}");
    };

    let Some(d) = ymd_p.get(6..=7) else {
        bail!("failed to get range 6..=7: {url}");
    };

    let day = Day{
        day: d.to_string(),
        root_url: url
    };
    Ok(day)
}

async fn scrape(
    client: &HttpClient, 
    a_tag: &Selector, 
    month: &mut Month,
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
        let day = into_day(url)?;
        if month.days.iter().find(|d| d.day == day.day).is_some() {
            continue;
        }
        month.days.push(day);
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
    let mut list = serde_json::from_str::<Vec<Month>>(&s)?;
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

        scrape(&client, &a_tag, month, &root).await?;

        sleep(interval).await;
    }

    let json = serde_json::to_string_pretty(&list)?;
    fs::write(SOURCE, json).await?;

    info!("done in {}milsecs", start.elapsed().as_millis());
    Ok(())
}