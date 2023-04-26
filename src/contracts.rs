use ethers::contract::abigen ;

// note: QuickswapV2 is a uniswap clone,so use it with quickswap address and uniwap declarations
abigen!(
    Arbitrageur,  "./data/abis/Arbitrageur.json" ;
    Ierc20,  "./data/abis/Ierc20.json" ;
    UniswapV3Factory, "./data/abis/UniswapV3Factory.json";
    UniswapV3Pool, "./data/abis/UniswapV3Pool.json" ;
    UniswapV3Quoter, "./data/abis/UniswapV3Quoter.json" ;
    UniswapV3QuoterV2,
    r#"[
      function quoteExactInputSingle() public override returns (uint256 amountOut, uint160 sqrtPriceX96After, uint32 initializedTicksCrossed, uint256 gasEstimate)
      function quoteExactInput() public override returns (uint256 amountOut, uint160[] sqrtPriceX96AfterList, uint32[] initializedTicksCrossedList, uint256 gasEstimate)
    ]"#;
    UniswapV2Factory, "./data/abis/UniswapV2Factory.json" ;
    UniswapV2Pair, "./data/abis/UniswapV2Pair.json" ;
    UniswapV2Library,
    r#"[
        function pairFor(address factory, address tokenA, address tokenB) internal pure returns (address pair)
        function getReserves(address factory, address tokenA, address tokenB) internal view returns (uint reserveA, uint reserveB)
        function quote(uint amountA, uint reserveA, uint reserveB) internal pure returns (uint amountB)
        function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) internal pure returns (uint amountOut)
        function getAmountsOut(uint amountIn, address[] memory path) internal view returns (uint[] memory amounts)
    ]"# ;
    QuickswapV3Factory,
    r#"[
        function poolByPair(address tokenA, address tokenB) external view returns (address pool)
    ]"#;
    QuickswapV3Quoter,
    r#"[
        function quoteExactInput(bytes memory path, uint256 amountIn) external returns (uint256 amountOut, uint16[] memory fees)
        function quoteExactInputSingle(address tokenIn, address tokenOut, uint256 amountIn, uint160 limitSqrtPrice) external returns (uint256 amountOut, uint16 fee)
    ]"#
);
