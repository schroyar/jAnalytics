use ethers::contract::abigen;

abigen!(
    JonesRouter,
    "src/abis/Router.json",
    derives(serde::Deserialize, serde::Serialize)
);
