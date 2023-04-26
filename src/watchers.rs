use crate::assets::Asset;
use crate::dexs::Dex;
use ethers::{
    providers::{Http, Provider},
    types::Address,
};
use std::sync::Arc;
use tabled::{Table, Tabled};

const AAVE_FEE: f64 = 0.05 ;
const UNISWAPV3_FEE : f64 = 0.05 ;
const QUICKSWAPV3_FEE : f64 = 0.10 ;

enum Direction {
    UniswapV3ToQuickswapV3 = 0,
    QuickswapV3ToUniswapV3 = 1,
}

#[derive(Tabled)]
struct TableData {
    #[tabled(rename = "dex name")]
    dex_name: String,

    #[tabled(rename = "pool name")]
    pool_name: String,

    #[tabled(rename = "pool address")]
    pool_address: String,

    #[tabled(rename = "trade asset balance")]
    pool_trade_asset_balance: f64,

    #[tabled(rename = "loan asset balance")]
    pool_loan_asset_balance: f64,

    // #[tabled(display_with = "Dex::get_trade_asset_sym")]
    #[tabled(rename = "trade asset out amount")]
    pool_trade_asset_out_amount: f64,

    #[tabled(rename = "loan asset out amount")]
    pool_loan_asset_out_amount: f64,
}

impl TableData {
    pub fn new(
        dex_name: String,
        pool_name: String,
        pool_address: String,
        pool_trade_asset_balance: f64,
        pool_loan_asset_balance: f64,
        pool_trade_asset_out_amount: f64,
        pool_loan_asset_out_amount: f64,
    ) -> TableData {
        TableData {
            dex_name,
            pool_name,
            pool_address,
            pool_trade_asset_balance,
            pool_loan_asset_balance,
            pool_trade_asset_out_amount,
            pool_loan_asset_out_amount,
        }
    }
}

struct PoolData {
    pub trade_balance: f64,
    pub loan_balance: f64,
    pub address: Address,
}

impl PoolData {
    pub fn from_pool_balance_out(tuple: (f64, f64, Address)) -> PoolData {
        return PoolData {
            trade_balance: tuple.0,
            loan_balance: tuple.1,
            address: tuple.2,
        };
    }
}

/// wacth price of dynamic price asset `asset_trade` and a
/// static price asset `asset_loan` in dexes `dex0` and `dex1`
/// if exist a trade oportunity request a flash loan of on
/// `asset_loan` to trade `asset_trade`
///
/// trade goes like this
/// loan `asset_loan`
/// buy `asset_trade` in one dex and sell it in another
/// get `asset_loan` back
pub struct Watcher {
    provider: Arc<Provider<Http>>,

    /// arbitrageur that will execute trades when detect a trade oportunity
    // arbitrageur_address: Arbitrageur<Arc<Provider<Http>>>,

    /// dynamic/variable price asset
    asset_trade: Asset,

    /// static price asset
    asset_loan: Asset,

    dex0: Dex,
    dex1: Dex,
}

impl Watcher {

    pub fn new(
        asset_trade: Asset,
        asset_loan: Asset,
        dex0: Dex,
        dex1: Dex,
        // arbitrageur: Address,
        provider: Arc<Provider<Http>>,
    ) -> Watcher {
        Watcher {
            asset_trade,
            asset_loan,
            dex0,
            dex1,
            provider,
            // arbitrageur: Arbitrageur::new(arbitrageur,&provider)
        }
    }

    pub fn from_pairs(
        assets: (Asset, Asset),
        dexs: (Dex, Dex),
        // arbitrageur: Arbitrageur<Arc<Provider<Http>>>,
        provider: Arc<Provider<Http>>,
    ) -> Watcher {
        Watcher {
            asset_trade: assets.0,
            asset_loan: assets.1,
            dex1: dexs.1,
            dex0: dexs.0,
            provider,
            // arbitrageur
        }
    }

    pub async fn watch(&self) -> Result<(), ()> {
        let Ok((asset_trade_out_amount_dex0,asset_trade_price_dex0)) = self.dex0.check_assets_price(&self.asset_trade, &self.asset_loan).await else {
            println!("could not get price of assets in dex0");
            return Err(())
        };
        let Ok((asset_trade_out_amount_dex1,asset_trade_price_dex1)) = self.dex1.check_assets_price(&self.asset_trade, &self.asset_loan).await else {
            println!("could not get price of assets in dex1");
            return Err(())
        };

        let direction = self.test_trade(
            &asset_trade_out_amount_dex0,
            &asset_trade_price_dex0,
            &asset_trade_out_amount_dex1,
            &asset_trade_price_dex1
        );

        let failed_state = String::from("failed");
        let state = if let Some(dir) = direction {
            self.trade(dir).await.unwrap_or(failed_state);
        } else {
            failed_state;
        };

        self.show(
            &asset_trade_out_amount_dex0,
            &asset_trade_price_dex0,
            &asset_trade_out_amount_dex1,
            &asset_trade_price_dex1,
        ).await ;

        return Ok(());
    }

    fn test_trade(&self,dex0_out_amount: &f64,price0: &f64,dex1_out_amount: &f64, price1: &f64) -> Option<Direction> {

        if price0 > price1 && Self::calc_roi(&dex1_out_amount,&dex0_out_amount) {

            println!("quick to uni tested trade");
            return Some(Direction::QuickswapV3ToUniswapV3);

        } else if price1 > price0 &&  Self::calc_roi(&dex0_out_amount,&dex1_out_amount) {

            println!("uni to quick tested trade");
            return Some(Direction::UniswapV3ToQuickswapV3);

        } else {

            return None;

        }

    }

    async fn trade(&self,direction: Direction) -> Result<String, ()> {

        let Ok((
            dex0_pool_data,
            dex1_pool_data
        )) = self.get_pool_data().await else {
            println!("err getting pool data");
            return Err(())
        };

        match direction {
            Direction::UniswapV3ToQuickswapV3 => {
                // call_arbitrageur(baseAsset,quoteAsset,quoteAssetAmount,direction,fee) ;
                return Ok(String::from("ok: uni to quick"));
            }
            Direction::QuickswapV3ToUniswapV3 => {
                return Ok(String::from("ok: quick to uni"));
            }
            _ => return Err(()),
        }
    }

    async fn show(
        &self,
        out_amount_dex0: &f64,
        price_dex0: &f64,
        out_amount_dex1: &f64,
        price_dex1: &f64,
    ) {
        let (asset_trade_sym, asset_loan_sym) = self.get_asset_syms();
        let (dex0_name, dex1_name) = self.get_dexs_names();
        let pool_name = format!("{asset_trade_sym}/{asset_loan_sym}");

        let Ok((
            dex0_pool_data,
            dex1_pool_data
        )) = self.get_pool_data().await else {
            println!("error getting pool data");
            return
        };

        let table = Table::new(vec![
            TableData::new(
                dex0_name.to_string(),
                pool_name.to_string(),
                dex0_pool_data.address.to_string(),
                dex0_pool_data.trade_balance.clone(),
                dex0_pool_data.loan_balance.clone(),
                out_amount_dex0.clone(),
                price_dex0.clone(),
            ),
            TableData::new(
                dex1_name.clone(),
                pool_name.clone(),
                dex1_pool_data.address.to_string(),
                dex1_pool_data.trade_balance.clone(),
                dex1_pool_data.loan_balance.clone(),
                out_amount_dex1.clone(),
                price_dex1.clone(),
            ),
        ])
        .with(tabled::Style::rounded())
        .to_string();

        println!("\n{}\n", table);
    }


    async fn get_pool_data(&self) -> Result<(PoolData, PoolData), ()> {
        let dex0_pool_data = PoolData::from_pool_balance_out(
            self.dex0
                .get_pool_balance(self.asset_loan.clone(), self.asset_trade.clone())
                .await,
        );

        let dex1_pool_data = PoolData::from_pool_balance_out(
            self.dex1
                .get_pool_balance(self.asset_loan.clone(), self.asset_trade.clone())
                .await,
        );

        return Ok((dex0_pool_data, dex1_pool_data));
    }

    #[inline]
    fn get_asset_syms(&self) -> (String, String) {
        return (self.asset_trade.symbol(), self.asset_loan.symbol());
    }

    #[inline]
    fn get_dexs_names(&self) -> (String, String) {
        return (self.dex0.name(), self.dex1.name());
    }

    #[inline]
    pub fn get_loan_asset_sym(&self) -> String {
        return self.asset_loan.symbol();
    }

    #[inline]
    pub fn get_trade_asset_sym(&self) -> String {
        return self.asset_trade.symbol();
    }

    #[inline]
    pub fn calc_roi(in_amount: &f64,out_amount: &f64) -> bool {
        let roi = ( out_amount - in_amount ) * 100.0 / in_amount ;
        return roi > AAVE_FEE + UNISWAPV3_FEE + QUICKSWAPV3_FEE

    }

    #[inline]
    pub fn call_arbitrageur(&self,direction: Direction) -> Result<(),()> {
        // let Ok(state) = arbitrageur(
        //      self.asset_trade,
        //      self.asset_loan,
        //      amount,
        //      direction,
        //      fee
        // ).call().await
        //else {
        //  return Err(())
        // }
        todo!()
    }

}
