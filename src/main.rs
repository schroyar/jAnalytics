use janalytics::get_yield;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("------------------------");
    get_yield::from_split_rewards().await?;
    println!("------------------------");
    get_yield::total_retention_redeem_glp_first_router().await?;
    println!("------------------------");
    get_yield::total_retention_redeem_glp_current_router().await?;
    println!("------------------------");

    Ok(())
}
