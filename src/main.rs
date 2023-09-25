use janalytics::get_yield;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    get_yield::from_split_rewards().await?;

    Ok(())
}
