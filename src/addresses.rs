use ethers::types::Address ;

const UNISWAPV3_FACTORY_ADDRESS: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
const UNISWAPV3_QUOTER_ADDRESS: &str = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6";
const UNISWAPV2_FACTORY_ADDRESS: &str = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";
const QUICKSWAPV3_FACTORY_ADDRESS: &str = "0x411b0fAcC3489691f28ad58c47006AF5E3Ab3A28";
const QUICKSWAPV3_QUOTER_ADDRESS: &str = "0xa15F0D7377B2A0C0c10db057f641beD21028FC89";
const QUICKSWAPV2_FACTORY_ADDRESS: &str = "0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32";

pub fn get_contract_addresses() -> Option<(Address, Address, Address, Address, Address, Address)> {
    let Ok(uniswapv3_factory_address) = UNISWAPV3_FACTORY_ADDRESS.parse::<Address>() else {
        return None
    } ;
    let Ok(uniswapv3_quoter_address) = UNISWAPV3_QUOTER_ADDRESS.parse::<Address>() else {
        return None
    } ;
    let Ok(uniswapv2_factory_address) = UNISWAPV2_FACTORY_ADDRESS.parse::<Address>() else {
        return None
    } ;
    let Ok(quickswapv3_factory_address) = QUICKSWAPV3_FACTORY_ADDRESS.parse::<Address>() else {
        return None
    } ;
    let Ok(quickswapv3_quoter_address) = QUICKSWAPV3_QUOTER_ADDRESS.parse::<Address>() else {
        return None
    } ;
    let Ok(quickswapv2_factory_address) = QUICKSWAPV2_FACTORY_ADDRESS.parse::<Address>() else {
        return None
    } ;

    return Some((
        uniswapv3_factory_address,
        uniswapv3_quoter_address,
        uniswapv2_factory_address,
        quickswapv3_factory_address,
        quickswapv3_quoter_address,
        quickswapv2_factory_address,
    ));
}
