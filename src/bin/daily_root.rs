use std::{env, time::Duration};
use anyhow::bail;
use tracing::{warn, info};
use tokio::{fs, time};
use scraper::{Html, Selector};
use reqwest::Client as HttpClient;
use mongodb::Client as MongoClient;
use suru_dev::{DailyProducts, MonthlyRoot, ProductDescription};

const SOURCE: &'static str = "json/root.json";

#[derive(Debug)]
pub struct Page {
    pub page: u32,
    pub url: String
}

fn scrape_pages(
    html: &Html,
    table_tag: &Selector,
    a_tag: &Selector,
    url_root: &str
) -> Vec<Page> {
    let mut page_list = vec![];

    for table in html.select(&table_tag) {
        for a in table.select(&a_tag) {
            let Some(url) = a.attr("href") else {
                warn!("skipping list item without href");
                continue;
            };
            
            let texts = a.text().collect::<Vec<&str>>();
            let page = match texts.len() {
                0 => {
                    warn!("skipping item without text");
                    continue;
                }
                1 => texts[0],
                _ => {
                    warn!("element has multiple texts: {texts:?}");
                    texts[0]
                }
            };

            let Ok(page) = page.parse() else {
                continue;
            };

            let p = Page{
                page,
                url: format!("{url_root}/{url}")
            };
            page_list.push(p);
        }
    }

    page_list
}

fn scrape_descriptions(
    html: &Html,
    ul_tag: &Selector,
    a_tag: &Selector,
) -> Vec<ProductDescription> {
    let mut desc_list = vec![];
    
    for ul in html.select(&ul_tag) {
        for a in ul.select(&a_tag) {
            let Some(url) = a.attr("href") else {
                warn!("skipping list item without href");
                continue;
            };

            let texts = a.text()
                .map(|s| s.trim())
                .collect::<Vec<&str>>();
            let description = match texts.len() {
                0 => {
                    warn!("skipping item without text");
                    continue;
                }
                1 => texts[0],
                _ => {
                    warn!("element has multiple texts: {texts:?}");
                    texts[0]
                }
            };
            
            let desc = ProductDescription{
                description: description.to_string(),
                url: url.to_string()
            };
            desc_list.push(desc);
        }
    }

    desc_list
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;

    let collection = MongoClient::with_uri_str(env::var("DB_URL")?).await?
        .database(&env::var("DB_NAME")?)
        .collection::<DailyProducts>("daily");

    let json = fs::read_to_string(SOURCE).await?;
    let root = serde_json::from_str::<Vec<MonthlyRoot>>(&json)?;

    let ul_tag = match Selector::parse("ul") {
        Ok(t) => t,
        Err(e) => bail!("{e}")
    };
    let table_tag = match Selector::parse("table") {
        Ok(t) => t,
        Err(e) => bail!("{e}")
    };
    let a_tag = match Selector::parse("a") {
        Ok(t) => t,
        Err(e) => bail!("{e}")
    };

    let url_root = env::var("URL_ROOT")?;
    let http_client = HttpClient::new();
    let interval = Duration::from_millis(750);
    let target_year = vec![2024];

    for m in root.iter() {
        if !target_year.contains(&m.year) {
            continue;
        }

        info!("{}/{}", m.year, m.month);

        for d in m.products.iter() {
            info!("day {}", d.day);
            let mut prod_list = vec![];

            let document = http_client.get(&d.url)
                .send().await?
                .text().await?;
            let html = Html::parse_document(&document);
            let page_list = scrape_pages(&html, &table_tag, &a_tag, &url_root);
            let mut desc_list = scrape_descriptions(&html, &ul_tag, &a_tag);
            prod_list.append(&mut desc_list);

            if page_list.len() > 1 {
                for p in page_list[1..].iter() {
                    info!("page {}", p.page);
    
                    let document = http_client.get(&p.url)
                        .send().await?
                        .text().await?;
                    let html = Html::parse_document(&document);
                    let mut desc_list = scrape_descriptions(&html, &ul_tag, &a_tag);
                    prod_list.append(&mut desc_list);
    
                    time::sleep(interval).await;
                }    
            }

            let daily_products = DailyProducts{
                year: m.year,
                month: m.month,
                day: d.day,
                products: prod_list 
            };
            collection.insert_one(daily_products).await?;
            
            info!("inserted into collection");
            time::sleep(interval).await;    
        }
    }

    Ok(())
}
