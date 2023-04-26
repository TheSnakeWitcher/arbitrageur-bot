use config::{self, Config};
use ethers::types::Address;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{collections::HashMap, fs};

// app
const APP_NAME: &str = "arbitrageur";
const APP_PREFIX: &str = "ARBITRAGEUR";

// config file
const CONFIG_FILE_DIR: &str = APP_NAME;
const CONFIG_FILE_NAME: &str = APP_NAME;
const CONFIG_FILE_EXTENSION: &str = "toml";
const CONFIG_FILE_DEV: &str = "/home/mr-papi/SoftwareCode/Projects/arbitrageur-bot/data/arbitrageur.conf.toml";

// defaults
const LOGFILE_KEY: &str = "logfile";
const LOGFILE_DEFAULT_VALUE: &str =
    "/home/mr-papi/SoftwareCode/Projects/arbitrageur-bot/arbitrageur.log";
const TRADE_ASSETS_KEY: &str = "trade";
const LOAN_ASSETS_KEY: &str = "loan";

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub rpc_url: String,
    pub db_url: String,
    pub logfile: String,
    pub assets: String,
}

pub fn new() -> Result<Configuration, ()> {
    let config_dir = dirs::config_dir().expect("failed to load XDG_CONFIG_DIR");

    let app_config_file = {
        let mut app_config_path = config_dir.join(CONFIG_FILE_DIR);
        app_config_path.push(CONFIG_FILE_NAME);
        app_config_path.set_extension(CONFIG_FILE_EXTENSION);
        app_config_path
    };

    let conf_builder = Config::builder()
        .set_default(LOGFILE_KEY, LOGFILE_DEFAULT_VALUE)
        .expect("failod to set default logfile")
        .add_source(config::File::from(app_config_file).required(false))
        .add_source(config::File::with_name(CONFIG_FILE_DEV).required(false))
        .add_source(config::Environment::with_prefix(APP_PREFIX));

    match conf_builder.build() {
        Ok(cfg) => {
            return Ok(cfg
                .try_deserialize::<Configuration>()
                .expect("error deserializing config"));
        }
        Err(e) => {
            println!("configuration error: {}", e);
            Err(())
        }
    }
}

pub fn get_assets(assets_path: &str) -> Result<(Vec<Address>, Vec<Address>), ()> {
    let Ok(assets) = fs::read_to_string(assets_path) else {
        return Err(())
    };

    let Ok(assets_hashmap) = serde_json::from_str::<HashMap<String,Value>>(&assets) else {
        return Err(())
    };

    let Some(trade_assets_hashmap) = assets_hashmap[TRADE_ASSETS_KEY].as_object() else {
        return Err(())
    };

    let Some(loan_assets_hashmap) = assets_hashmap[LOAN_ASSETS_KEY].as_object() else {
        return Err(())
    };

    let trade_assets: Vec<Address> = trade_assets_hashmap
        .values()
        .filter_map(|address| parse_address(address))
        .collect();

    let loan_assets: Vec<Address> = loan_assets_hashmap
        .values()
        .filter_map(|address| parse_address(address))
        .collect();

    return Ok((trade_assets, loan_assets));
}

fn parse_address(address: &Value) -> Option<Address> {
    let Some(address_str) = address.as_str() else {
        return None
    };

    let Some(parsed_address) = address_str.parse::<Address>().ok() else {
        return None
    };

    return Some(parsed_address);
}
