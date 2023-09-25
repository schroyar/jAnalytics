use crate::bindings::jones_glp_rewards_distributor::JonesGlpRewardsDistributorEvents;
use crate::bindings::jones_router::JonesRouterEvents;
use crate::bindings::jones_router_new::JonesRouterNewEvents;
use chrono::format;
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

// We had 2 routers, this is the first one
const GENESIS_FIRST_ROUTER: u64 = 55_870_909;
const END_FIRST_ROUTER: u64 = 61_636_421;

// And this is the second one, that is currently being used, so end block will be current block - buffer
const GENESIS_CURRENT_ROUTER: u64 = 61_578_908;

const GENESIS_REWARD_DISTRIBUTOR: u64 = 55_870_678;

// Set according to try and error
const MAX_BLOCK_RANGE: u64 = 10_000_000;

// We had 2 routers
pub async fn total_retention_redeem_glp_first_router() -> Result<(), anyhow::Error> {
    // Create a new `ankr` provider instance for the Arbitrum node
    // Not putting in an env since its public anyway
    let rpc = Provider::try_from("https://rpc.ankr.com/arbitrum").unwrap();

    // This one was deprecated at Feb 17th
    let first_router = "0x01aD96292cdc627307817c428562226fd905AEc2";

    let event_sig = "RedeemGlp(address,uint256,uint256,uint256,bool)";
    let event_sig_redeem_stable = "RedeemStable(address,uint256,uint256,uint256,bool)";
    let event_sig_redeem_glp_eth = "RedeemGlpEth(address,uint256,uint256,uint256,uint256)";

    let filter_ = Filter::new()
        .address(first_router.parse::<Address>().unwrap())
        .event(event_sig)
        .from_block(BlockNumber::Number(GENESIS_FIRST_ROUTER.into()))
        .to_block(BlockNumber::Number(END_FIRST_ROUTER.into()));

    let filter_redeem_stable = Filter::new()
        .address(first_router.parse::<Address>().unwrap())
        .event(event_sig_redeem_stable)
        .from_block(BlockNumber::Number(GENESIS_FIRST_ROUTER.into()))
        .to_block(BlockNumber::Number(END_FIRST_ROUTER.into()));

    let filter_redeem_glp_eth = Filter::new()
        .address(first_router.parse::<Address>().unwrap())
        .event(event_sig_redeem_glp_eth)
        .from_block(BlockNumber::Number(GENESIS_FIRST_ROUTER.into()))
        .to_block(BlockNumber::Number(END_FIRST_ROUTER.into()));

    let logs = rpc.get_logs(&filter_).await?;
    let logs_redeem_stable = rpc.get_logs(&filter_redeem_stable).await?;
    let logs_redeem_glp_eth = rpc.get_logs(&filter_redeem_glp_eth).await?;

    // Initialize a variable to hold the cumulative sum of `glp_rewards`, `jones_rewards`, and `stable_rewards`
    let mut cumulative_u256: U256 = 0.into();
    let mut cumulative_redeem_stable: U256 = 0.into();
    let mut cumulative_redeem_glp_eth: U256 = 0.into();

    for log in logs {
        let topics = log.clone().topics;
        let data = log.clone().data;

        let mut logger;

        logger = JonesRouterEvents::decode_log(&RawLog {
            topics,
            data: data.to_vec(),
        })
        .unwrap();

        if let JonesRouterEvents::RedeemGlpFilter(e) = logger {
            let x: U256 = e.eth_retentions;

            // Add the sum to the cumulative sum
            cumulative_u256 = cumulative_u256 + x;
        }
    }

    for log_redeem_stable in logs_redeem_stable {
        let topics = log_redeem_stable.clone().topics;
        let data = log_redeem_stable.clone().data;

        let mut logger;

        logger = JonesRouterEvents::decode_log(&RawLog {
            topics,
            data: data.to_vec(),
        })
        .unwrap();

        if let JonesRouterEvents::RedeemStableFilter(e) = logger {
            let x: U256 = e.real_retentions;

            // Add the sum to the cumulative sum
            cumulative_redeem_stable = cumulative_redeem_stable + x;
        }
    }

    for log_redeem_glp_eth in logs_redeem_glp_eth {
        let topics = log_redeem_glp_eth.clone().topics;
        let data = log_redeem_glp_eth.clone().data;

        let mut logger;

        logger = JonesRouterEvents::decode_log(&RawLog {
            topics,
            data: data.to_vec(),
        })
        .unwrap();

        if let JonesRouterEvents::RedeemGlpEthFilter(e) = logger {
            let x: U256 = e.eth_retentions;

            // Add the sum to the cumulative sum
            cumulative_redeem_glp_eth = cumulative_redeem_glp_eth + x;
        }
    }

    println!(
        "From block {} to {}\nRetention Redeem GLP: {} ETH\nRetention Redeem Stable: {} USDC\nRetention Redeem GLP ETH: {} ETH",
        GENESIS_FIRST_ROUTER.to_string().green(),
        END_FIRST_ROUTER.to_string().green(),
        format_units(cumulative_u256, 18)
            .unwrap()
            .to_string()
            .purple(),
        format_units(cumulative_redeem_stable, 6)
            .unwrap()
            .to_string()
            .purple(),
        format_units(cumulative_redeem_glp_eth, 18)
            .unwrap()
            .to_string()
            .purple()
    );

    Ok(())
}

pub async fn total_retention_redeem_glp_current_router() -> Result<(), anyhow::Error> {
    // Create a new `ankr` provider instance for the Arbitrum node
    // Not putting in an env since its public anyway
    let rpc = Provider::try_from("https://rpc.ankr.com/arbitrum").unwrap();

    // Get the latest block number
    let mut latest_block = rpc.get_block_number().await?;

    // - 1000 blocks just to be sure, rewards shouldnt change a lot in 1000 blocks
    latest_block = latest_block - U64::from_dec_str("1000")?;

    let block_diff = latest_block - GENESIS_CURRENT_ROUTER;

    // Parse the distributor address as an `Address` instance
    let router = "0x2F43c6475f1ecBD051cE486A9f3Ccc4b03F3d713"
        .parse::<Address>()
        .unwrap();

    let mut start_block;
    let mut end_block;

    let iterations: u64 = block_diff.as_u64() / MAX_BLOCK_RANGE;

    let mut cumulative_total_redeem_glp: U256 = 0.into();
    let mut cumulative_total_redeem_stable: U256 = 0.into();

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
        let filter_redeem_glp = Filter::new()
            .address(router)
            .from_block(BlockNumber::Number(start_block.into()))
            .to_block(BlockNumber::Number(end_block.into()))
            .event("RedeemGlp(address,uint256,uint256,uint256,address,uint256,bool)");

        let filter_redeem_stable = Filter::new()
            .address(router)
            .from_block(BlockNumber::Number(start_block.into()))
            .to_block(BlockNumber::Number(end_block.into()))
            .event("RedeemStable(address,uint256,uint256,uint256,bool)");

        // Retrieve the logs that match the filter from the Arbitrum node
        let logs_redeem_glp = rpc.get_logs(&filter_redeem_glp).await?;
        let logs_redeem_stable = rpc.get_logs(&filter_redeem_stable).await?;

        // Initialize a variable to hold the decoded event log
        let mut logger;

        // Initialize a variable to hold the cumulative sum of `glp_rewards`, `jones_rewards`, and `stable_rewards`
        let mut cumulative_u256: U256 = 0.into();
        let mut cumulative_u256_redeem_stable: U256 = 0.into();

        // Iterate over each log and decode the `SplitRewards` event log
        for log in logs_redeem_glp {
            let topics = log.clone().topics;
            let data = log.clone().data;

            logger = JonesRouterNewEvents::decode_log(&RawLog {
                topics,
                data: data.to_vec(),
            })
            .unwrap();

            // If the decoded event log is a `SplitRewards` event, calculate the sum of `glp_rewards`, `jones_rewards`, and `stable_rewards`
            if let JonesRouterNewEvents::RedeemGlpFilter(e) = logger {
                let x: U256 = e.eth_retentions;

                // Add the sum to the cumulative sum
                cumulative_u256 = cumulative_u256 + x;
            }
        }

        for log_redeem_stable in logs_redeem_stable {
            let topics = log_redeem_stable.clone().topics;
            let data = log_redeem_stable.clone().data;

            logger = JonesRouterNewEvents::decode_log(&RawLog {
                topics,
                data: data.to_vec(),
            })
            .unwrap();

            // If the decoded event log is a `SplitRewards` event, calculate the sum of `glp_rewards`, `jones_rewards`, and `stable_rewards`
            if let JonesRouterNewEvents::RedeemStableFilter(e) = logger {
                let x: U256 = e.real_retentions;

                // Add the sum to the cumulative sum
                cumulative_u256_redeem_stable = cumulative_u256_redeem_stable + x;
            }
        }

        cumulative_total_redeem_glp = cumulative_total_redeem_glp + cumulative_u256;
        cumulative_total_redeem_stable =
            cumulative_total_redeem_stable + cumulative_u256_redeem_stable;

        // Print the cumulative sum to the console

        println!(
            "From block {} to block {}: \nRetention Redeem GLP: {} ETH\nRetention Redeem Stable: {} USDC",
            start_block.to_string().green(),
            end_block.to_string().green(),
            format_units(cumulative_u256, 18)
                .unwrap()
                .to_string()
                .purple(),
            format_units(cumulative_u256_redeem_stable, 6)
                .unwrap()
                .to_string()
                .purple()
        );

        i += 1;
    }

    println!(
        "Total retention for RedeemGlp (2nd router): {} ETH\nTotal retention for RedeemStable (2nd router): {}",
        format_units(cumulative_total_redeem_glp, 18)
            .unwrap()
            .to_string()
            .purple(),
        format_units(cumulative_total_redeem_stable, 6)
            .unwrap()
            .to_string()
            .purple()
    );

    Ok(())
}

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
        /*
        println!(
            "From block {} to block {}: {} ETH",
            start_block.to_string().red(),
            end_block.to_string().red(),
            format_units(cumulative_u256, 18)
                .unwrap()
                .to_string()
                .green()
        );
        */

        i += 1;
    }

    println!(
        "Total rewards for SplitRewards: {} ETH",
        format_units(cumulative_total, 18)
            .unwrap()
            .to_string()
            .purple()
    );

    Ok(())
}

pub async fn from_compound() -> Result<(), anyhow::Error> {
    Ok(())
}
