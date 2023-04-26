use crate::assets::{Asset, Ierc20};
use ethers::{
    contract::abigen,
    providers::{Http, Middleware, Provider},
    types::{Address, Chain, U256},
    utils::{format_units, parse_units},
};
use std::sync::Arc;

const UNISWAPV3_FEES_LENGTH: usize = 4 ;
const UNISWAPV3_FEES: [u32; UNISWAPV3_FEES_LENGTH] = [100,500, 3000, 10000];

// NOTE: How to query 
// UniswapV2Pair::new(pair_address,Arc::new(&provider)) ;
// let pair = pairFor(factory,token0,token1)
// let (reserve0,reserve1) = pair.get_reserves(factory,token0,token1).call().await
// let amount_out =  get_amount_out(amount_in,reserve0,reserve1)
// let amount_out =  get_amounts_out(amount_in,path)

abigen!(
    UniswapV3Factory, "./data/abis/UniswapV3Factory.json";
    UniswapV3Quoter, "./data/abis/UniswapV3Quoter.json" ;
    QuickswapV3Factory,
    r#"[
        function poolByPair(address tokenA, address tokenB) external view returns (address pool)
    ]"#;
    QuickswapV3Quoter,
    r#"[
        function quoteExactInput(bytes memory path, uint256 amountIn) external returns (uint256 amountOut, uint16[] memory fees)
        function quoteExactInputSingle(address tokenIn, address tokenOut, uint256 amountIn, uint160 limitSqrtPrice) external returns (uint256 amountOut, uint16 fee)
    ]"#;
    // Arbitrageur, "./data/abis/Arbitrageur.json";
);

enum UniswapV3Fee {
    Lowest = 100,
    Low = 500,
    Medium = 3000,
    High = 10_000,
}

// NOTE: fees:
// 0.1%-0.15% on quickswapV3 
// 0.3% on uniswapv2,sushiswap,quickswapV2 
#[derive(Clone)]
pub enum Dex {
    UniswapV3 {
        name: String,
        factory: Address,
        quoter: Address,
        provider: Arc<Provider<Http>>,
    },
    QuickswapV3 {
        name: String,
        factory: Address,
        quoter: Address,
        provider: Arc<Provider<Http>>,
    },
    // UniswapV2(Address),
    // QuickswapV2(Address),
}

impl Dex {
    pub fn name(&self) -> String {
        match self {
            Self::UniswapV3 {
                name,
                factory,
                quoter,
                provider,
            } => return name.clone(),
            Self::QuickswapV3 {
                name,
                factory,
                quoter,
                provider,
            } => return name.clone(),
        }
    }

    pub async fn check_assets_price(
        &self,
        asset_in: &Asset,
        asset_out: &Asset,
    ) -> Result<(f64, f64), ()> {
        match self {
            Self::UniswapV3 {
                name,
                factory,
                quoter,
                provider,
            } => {
                let (quoter, in_amount) = (
                    UniswapV3Quoter::new(*quoter, provider.clone()),
                    U256::from(parse_units(1, asset_in.decimals()).unwrap()),
                );
                let out_amount = quoter
                    .quote_exact_input_single(
                        asset_in.address(),
                        asset_out.address(),
                        UniswapV3Fee::Low as u32,
                        in_amount,
                        U256::zero(),
                    )
                    .call()
                    .await
                    .unwrap();
                let out_amt: f64 = format_units(out_amount.clone(), asset_out.decimals())
                    .unwrap()
                    .parse()
                    .unwrap();
                let price = {
                    let in_amt: f64 = format_units(in_amount, asset_in.decimals())
                        .unwrap()
                        .parse()
                        .unwrap();
                    in_amt / out_amt
                };

                return Ok((out_amt, price));
            }
            Self::QuickswapV3 {
                name,
                factory,
                quoter,
                provider,
            } => {
                let (quoter, in_amount) = (
                    QuickswapV3Quoter::new(*quoter, provider.clone()),
                    U256::from(parse_units(1, asset_in.decimals()).unwrap()),
                );
                let (out_amount, fee) = quoter
                    .quote_exact_input_single(
                        asset_in.address(),
                        asset_out.address(),
                        in_amount,
                        U256::zero(),
                    )
                    .call()
                    .await
                    .unwrap();
                let out_amt: f64 = format_units(out_amount.clone(), asset_out.decimals())
                    .unwrap()
                    .parse()
                    .unwrap();
                let price = {
                    let in_amt: f64 = format_units(in_amount, asset_in.decimals())
                        .unwrap()
                        .parse()
                        .unwrap();
                    in_amt / out_amt
                };

                return Ok((out_amt, price));
            }
        }
    }

    pub async fn get_pool_balance(&self, asset_trade: Asset, asset_loan: Asset) -> (f64, f64,Address) {
        match self {
            Self::UniswapV3 {
                name,
                factory,
                quoter,
                provider,
            } => {
                let pool_address = {
                    let factory_contract = UniswapV3Factory::new(factory.clone(), provider.clone());
                    let pool_address = factory_contract
                        .get_pool(
                            asset_trade.address(),
                            asset_loan.address(),
                            UniswapV3Fee::Low as u32,
                        )
                        .call()
                        .await
                        .unwrap();
                    pool_address
                };
                let (asset_trade_balance, asset_loan_balance) = (
                    asset_trade
                        .contract
                        .balance_of(pool_address.clone())
                        .call()
                        .await
                        .unwrap(),
                    asset_loan
                        .contract
                        .balance_of(pool_address.clone())
                        .call()
                        .await
                        .unwrap(),
                );
                let (asset_trade_balance_out, asset_loan_balance_out) = (
                    format_units(asset_trade_balance, asset_trade.decimals())
                        .unwrap()
                        .parse::<f64>()
                        .unwrap(),
                    format_units(asset_loan_balance, asset_loan.decimals())
                        .unwrap()
                        .parse::<f64>()
                        .unwrap(),
                );
                return (asset_trade_balance_out, asset_loan_balance_out,pool_address);
            }
            Self::QuickswapV3 {
                name,
                factory,
                quoter,
                provider,
            } => {
                let pool_address = {
                    let factory_contract =
                        QuickswapV3Factory::new(factory.clone(), provider.clone());
                    let pool_address = factory_contract
                        .pool_by_pair(asset_trade.address(), asset_loan.address())
                        .call()
                        .await
                        .unwrap();
                    pool_address
                };
                let (asset_trade_balance, asset_loan_balance) = (
                    asset_trade
                        .contract
                        .balance_of(pool_address.clone())
                        .call()
                        .await
                        .unwrap(),
                    asset_loan
                        .contract
                        .balance_of(pool_address.clone())
                        .call()
                        .await
                        .unwrap(),
                );
                let (asset_trade_balance_out, asset_loan_balance_out) = (
                    format_units(asset_trade_balance, asset_trade.decimals())
                        .unwrap()
                        .parse::<f64>()
                        .unwrap(),
                    format_units(asset_loan_balance, asset_loan.decimals())
                        .unwrap()
                        .parse::<f64>()
                        .unwrap(),
                );
                return (asset_trade_balance_out, asset_loan_balance_out, pool_address);
            }
        }
    }
}
