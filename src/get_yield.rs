use crate::bindings::jones_glp_rewards_distributor::JonesGlpRewardsDistributorEvents;
use colored::Colorize;
use ethers::contract::EthLogDecode;
use ethers::types::U64;
use ethers::utils::format_units;
use ethers::{
    abi::RawLog,
    core::types::{BlockNumber, Filter},
    providers::{Middleware, Provider},
    types::{Address, U256},
};

const GENESIS_REWARD_DISTRIBUTOR: u64 = 55_870_678;
// Set according to try and error
const MAX_BLOCK_RANGE: u64 = 10_000_000;

pub async fn from_split_rewards() -> Result<(), anyhow::Error> {
    // Create a new `ankr` provider instance for the Arbitrum node
    // Not putting in an env since its public anyway
    let rpc = Provider::try_from("https://rpc.ankr.com/arbitrum").unwrap();

    // Get the latest block number
    let mut latest_block = rpc.get_block_number().await?;

    // - 1000 blocks just to be sure, rewards shouldnt change a lot in 1000 blocks
    latest_block = latest_block - U64::from_dec_str("1000")?;

    let block_diff = latest_block - GENESIS_REWARD_DISTRIBUTOR;

    // Parse the distributor address as an `Address` instance
    let distributor = "0xda04B5F54756774AD405DE499bB5100c80980a12"
        .parse::<Address>()
        .unwrap();

    let mut start_block;
    let mut end_block;

    let iterations: u64 = block_diff.as_u64() / MAX_BLOCK_RANGE;

    let mut cumulative_total: U256 = 0.into();
    let mut i = 0;

    // Kinda of a manual round up
    while i < (iterations + 1) {
        // Wait for 10 seconds before each iteration
        // Doing this just to dont get the block range too wide ankr error please mi familia
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        start_block = GENESIS_REWARD_DISTRIBUTOR + (i * MAX_BLOCK_RANGE);
        end_block = start_block + MAX_BLOCK_RANGE;

        // Dont try scan rewards in future kek
        if end_block > latest_block.as_u64() {
            end_block = latest_block.as_u64();
        }

        // Create a new filter to retrieve logs for the `SplitRewards` event from the distributor address
        let filter_ = Filter::new()
            .address(distributor)
            .from_block(BlockNumber::Number(start_block.into()))
            .to_block(BlockNumber::Number(end_block.into()))
            .event("SplitRewards(uint256,uint256,uint256)");

        // Retrieve the logs that match the filter from the Arbitrum node
        let logs = rpc.get_logs(&filter_).await?;

        // Initialize a variable to hold the decoded event log
        let mut logger;

        // Initialize a variable to hold the cumulative sum of `glp_rewards`, `jones_rewards`, and `stable_rewards`
        let mut cumulative_u256: U256 = 0.into();

        // Iterate over each log and decode the `SplitRewards` event log
        for log in logs {
            let topics = log.clone().topics;
            let data = log.clone().data;

            logger = JonesGlpRewardsDistributorEvents::decode_log(&RawLog {
                topics,
                data: data.to_vec(),
            })
            .unwrap();

            // If the decoded event log is a `SplitRewards` event, calculate the sum of `glp_rewards`, `jones_rewards`, and `stable_rewards`
            if let JonesGlpRewardsDistributorEvents::SplitRewardsFilter(e) = logger {
                let x: U256 = e.glp_rewards + e.jones_rewards + e.stable_rewards;

                // Add the sum to the cumulative sum
                cumulative_u256 = cumulative_u256 + x;
            }
        }

        cumulative_total = cumulative_total + cumulative_u256;

        // Print the cumulative sum to the console
        println!(
            "From block {} to block {}: {}",
            start_block.to_string().red(),
            end_block.to_string().red(),
            format_units(cumulative_u256, 18)
                .unwrap()
                .to_string()
                .green()
        );

        i += 1;
    }

    println!(
        "Total rewards: {}",
        format_units(cumulative_total, 18)
            .unwrap()
            .to_string()
            .purple()
    );

    Ok(())
}
