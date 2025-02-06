use std::env;
use tracing::info;
use mongodb::Client;
use tokio::fs;
use suru_dev::MonthlyRoot;

const SOURCE: &'static str = "json/root.json";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;

    let db_url = env::var("DB_URL")?;
    let mongo = Client::with_uri_str(db_url).await?;
    let db_name = env::var("DB_NAME")?;
    let db = mongo.database(&db_name);
    let collection = db.collection::<MonthlyRoot>("monthly");

    let json = fs::read_to_string(SOURCE).await?;
    let list = serde_json::from_str::<Vec<MonthlyRoot>>(&json)?;

    let result = collection.insert_many(list).await?;
    info!("inserted {} items", result.inserted_ids.len());
    Ok(())
}
