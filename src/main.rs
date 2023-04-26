// TODO: check different fees for uniswap v3
// TODO: arbitrageur contract instance
// TODO: check price on uniswapV2/quickswapV2(factory/pools or library)
// TODO: swap on uniswapV2 and quickswapV2
// TODO: redis db to save asset data
// TODO: check about sqrt price https://ethereum.stackexchange.com/questions/98685/computing-the-uniswap-v3-pair-price-from-q64-96-number
// TODO: diff between &(type1,type2) and (&type1,&type2)
// TODO: search book "what things are called"
mod addresses;
mod assets;
mod aux;
mod configuration;
mod contracts;
mod dexs;
mod db;
mod watchers;

use assets::Asset;
use ethers::providers::{Http, Middleware, Provider};
use std::{sync::Arc, thread, time};
use watchers::Watcher;
use redis::{Client,Connection};

const POLLING_INTERVAL: u64 = 15_000;

#[tokio::main]
async fn main() -> Result<(), ()> {
    ////////////////////////////////////////////////////////////////////////////
    // configuration
    ////////////////////////////////////////////////////////////////////////////

    let Ok(conf) = configuration::new() else {
        println!("error initializing configuration");
        return Err(())
    };

    let Ok(provider) = Provider::<Http>::try_from(conf.rpc_url) else {
        println!("error initializing provider");
        return Err(())
    };

    let Ok((trade_assets_addresses,loan_assets_addresses)) = configuration::get_assets(&conf.assets) else {
        println!("error getting assets");
        return Err(())
    };

    let Ok(mut db_conn) = db::init(&conf.db_url).await else {
        println!("error initializing db");
        return Err(())
    };

    ////////////////////////////////////////////////////////////////////////////
    // asset
    ////////////////////////////////////////////////////////////////////////////

    println!("init assets...");
    let trade_assets: Vec<Asset> = assets::vec_from_addresses(trade_assets_addresses, &provider,&mut db_conn).await;
    let loan_assets: Vec<Asset> = assets::vec_from_addresses(loan_assets_addresses, &provider,&mut db_conn).await;

    println!("loan assets:\n");
    loan_assets
        .iter()
        .for_each(|loan_asset| println!("{}", loan_asset.symbol));
    println!("trade assets:\n");
    trade_assets
        .iter()
        .for_each(|trade_asset| println!("{}", trade_asset.symbol));

    let assets_pairs: Vec<(Asset, Asset)> =
        assets::pairs_from_addresses(&trade_assets, &loan_assets);

    ////////////////////////////////////////////////////////////////////////////
    // dexs
    ////////////////////////////////////////////////////////////////////////////

    println!("init dexs...");
    let Some((
        uniswapv3_factory_address,
        uniswapv3_quoter_address,
        uniswapv2_factory_address,
        quickswapv3_factory_address,
        quickswapv3_quoter_address,
        quickswapv2_factory_address,
    )) = addresses::get_contract_addresses() else {
        return Err(())
    };

    let (uni, quick) = (
        dexs::Dex::UniswapV3 {
            name: String::from("uniswap"),
            factory: uniswapv3_factory_address,
            quoter: uniswapv3_quoter_address,
            provider: Arc::new(provider.clone()),
        },
        dexs::Dex::QuickswapV3 {
            name: String::from("quickswap"),
            factory: quickswapv3_factory_address,
            quoter: quickswapv3_quoter_address,
            provider: Arc::new(provider.clone()),
        },
    );

    ////////////////////////////////////////////////////////////////////////////
    // watchers
    ////////////////////////////////////////////////////////////////////////////

    println!("init watchers...");
    let mut watchers_list: Vec<Watcher> = assets_pairs
        .iter()
        .map(|pair: &(Asset, Asset)| {
            return Watcher::from_pairs(
                pair.clone(),
                (uni.clone(), quick.clone()),
                Arc::new(provider.clone()),
            );
        })
        .collect();

    println!("init watch loop");
    loop {
        for watcher in watchers_list.iter_mut() {
            if let Err(err) = watcher.watch().await {
                println!("error ocurred: {:?}", err);
                continue;
            } 
        }
        println!("iteration completed...");
        thread::sleep(time::Duration::from_millis(POLLING_INTERVAL));
    }
}
