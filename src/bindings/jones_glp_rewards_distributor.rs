use ethers::contract::abigen;

abigen!(
    JonesGlpRewardsDistributor,
    "src/abis/JonesGlpRewardDistributor.json",
    derives(serde::Deserialize, serde::Serialize)
);
