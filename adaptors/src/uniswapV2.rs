pub use uniswapV2_mod::*;

mod uniswapV2_mod {

    use ethers::{
        providers::Provider,
        types::Address,
    };

    struct TokenPair {
        token0: Address,
        token1: Address,
        lptoken: Address
    }

    struct ArbitrageFinger {
        router: Address,
        tokens: Vec<Address>,
        token_pairs: Vec<TokenPair>,
    }

    #[test]
    fn test() {
        println!("AAA");
    }
    // Add token == node
    // getCurrentPrice
    // Query all price pairs for given tokens, create graph, process in arbitrage engine
}