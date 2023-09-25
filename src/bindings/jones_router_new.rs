use ethers::contract::abigen;

abigen!(
    JonesRouterNew,
    "src/abis/RouterNew.json",
    derives(serde::Deserialize, serde::Serialize)
);
