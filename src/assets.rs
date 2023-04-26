use ethers::{
    contract::abigen,
    providers::{Http, Middleware, Provider},
    types::Address,
};
use redis::{Commands, Connection};
use std::sync::Arc;

abigen!(
    Ierc20,"./data/abis/Ierc20.json" ;
);

#[derive(Clone)]
pub struct Asset {
    pub contract: Ierc20<Provider<Http>>,
    pub address: Address,
    pub symbol: String,
    pub decimals: u32,
}

impl Asset {
    pub async fn fromt_db_or_contract(
        address: &Address,
        provider: &Provider<Http>,
        conn: &mut Connection,
    ) -> Result<Asset, ()> {
        if let Err(_) = conn.keys::<&[u8], String>(address.as_bytes()) {
            let Ok(asset) = Self::from_contract(address, provider).await else {
                return Err(())
            };

            conn.hset::<&[u8], &str, &str, bool>(address.as_bytes(), "symbol", &asset.symbol);
            conn.hset::<&[u8], &str, &u32, bool>(address.as_bytes(), "decimals", &asset.decimals);
            redis::cmd("SAVE").execute(conn);

            return Ok(asset);
        }

        let Ok(symbol) = conn.hget(address.as_bytes(),"symbol") else {
            return Err(())
        };

        let Ok(decimals) = conn.hget(address.as_bytes(),"decimals") else {
            return Err(())
        };

        let contract = Ierc20::new(address.clone(), Arc::new(provider.clone()));

        return Ok(Asset {
            symbol,
            decimals,
            contract,
            address: *address,
        });
    }

    pub async fn from_contract(address: &Address, provider: &Provider<Http>) -> Result<Asset, ()> {
        let contract = Ierc20::new(*address, Arc::new(provider.clone()));

        let Ok(symbol ) = contract.symbol().call().await else {
            return Err(())
        };

        let Ok(decimals) = contract.decimals().call().await else {
            return Err(())
        };

        return Ok(Asset {
            contract,
            symbol,
            address: address.clone(),
            decimals: decimals as u32,
        });
    }

    pub fn address(&self) -> Address {
        return self.address.clone();
    }

    pub fn symbol(&self) -> String {
        return self.symbol.clone();
    }

    pub fn decimals(&self) -> u32 {
        return self.decimals.clone();
    }
}

pub async fn vec_from_addresses(
    asset_addresses: Vec<Address>,
    provider: &Provider<Http>,
    conn: &mut Connection,
) -> Vec<Asset> {
    let mut assets = Vec::<Asset>::with_capacity(asset_addresses.len());
    for address in asset_addresses.iter() {
        // NOTE: last changed line
        let Ok(asset) = Asset::fromt_db_or_contract(address,&provider,conn).await else {
            continue
        };
        assets.push(asset);
    }
    return assets;
}

pub fn pairs_from_addresses(assets0: &Vec<Asset>, assets1: &Vec<Asset>) -> Vec<(Asset, Asset)> {
    // trade_assets.iter().cartesian_product(loan_assets.iter())
    // (0..3).flat_map(|i| (0..4).map(move |j| (i, j)))
    let assets_pairs: Vec<(Asset, Asset)> = assets0
        .iter()
        .flat_map(|asset0| {
            assets1
                .iter()
                .map(move |asset1| (asset0.clone(), asset1.clone()))
        })
        .collect();
    return assets_pairs;
}
