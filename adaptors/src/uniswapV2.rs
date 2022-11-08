#![allow(dead_code, unused)]

pub use uniswapV2_mod::*;

use ethers::{
    providers::{Provider, JsonRpcClient},
    contract::Contract,
    types::Address,
    core::abi::Abi
};

use std::{
    fs::File,
    sync::Arc,
};

use eyre::Result;
use arbitrage_engine::utils::logger::{logText, logObject};
use futures::future::join_all;
use super::utils::{is_zero_address, are_addresses_equal};

mod uniswapV2_mod {
    use super::{
        File,
        Arc,
        Abi,
        Address,
        Provider,
        Contract,
        JsonRpcClient,
        Result,
        logText,
        logObject,
        join_all,
        is_zero_address,
        are_addresses_equal
    };

    #[derive(Debug)]
    struct TokenPair {
        token0: Address,
        token1: Address,
        lptoken: Address
    }

    #[derive(Debug)]
    struct UniswapV2ArbitrageFinder<P: JsonRpcClient> {
        factory: Contract<Arc<Provider<P>>>,
        router: Contract<Arc<Provider<P>>>,
        tokens: Vec<Address>,
        token_pairs: Vec<TokenPair>,
        provider: Arc<Provider<P>>
    }

    impl<P: JsonRpcClient + 'static> UniswapV2ArbitrageFinder<P> {
        pub fn new(factory_address: Address, router_address: Address, provider_: Provider<P>) -> Result<Self,> {
            let factory_abi_file = File::open("src/abis/UniswapV2Factory.json")?;
            let router_abi_file = File::open("src/abis/UniswapV2Router.json")?;
            let factory_abi: Abi = serde_json::from_reader(factory_abi_file)?;
            let router_abi: Abi = serde_json::from_reader(router_abi_file)?;
            let wrapped_provider = Arc::new(provider_);
            let factory: Contract<Arc<Provider<P>>> = Contract::new(factory_address, factory_abi, Arc::clone(&wrapped_provider));
            let router: Contract<Arc<Provider<P>>> = Contract::new(router_address, router_abi, Arc::clone(&wrapped_provider));

            Ok(UniswapV2ArbitrageFinder {         
                factory,
                router,
                tokens: Vec::new(),
                token_pairs: Vec::new(),
                provider: Arc::clone(&wrapped_provider) 
            })
        }

        pub async fn add_token(&mut self, token_address: Address) -> Result<()>  {
            // For each pre-existing token, attempt to find the pair
            let mut get_pair_futures = Vec::new();

            for token in self.tokens.iter() {
                let get_pair_future = self.factory
                    .method::<_, Address>("getPair", (*token, token_address))?
                    .call()
                    .await;

                get_pair_futures.push(tokio::spawn(async move {get_pair_future}));
            }

            let wrapped_pairs = join_all(get_pair_futures).await;

            for (i, token) in self.tokens.iter().enumerate() {
                if wrapped_pairs[i].is_ok() && wrapped_pairs[i].as_ref().unwrap().as_ref().is_ok() {
                    let pair = wrapped_pairs[i].as_ref().unwrap().as_ref().unwrap();
                    if !is_zero_address(*pair) {
                        self.token_pairs.push(TokenPair{
                            token0: *token,
                            token1: token_address,
                            lptoken: *pair
                        });
                    }
                }
            }

            self.tokens.push(token_address);
            Ok(())
        }

        pub fn remove_token(&mut self, token_address: Address) {
            let mut token_found = false;
            let mut tokens_index: usize = 0;
            let mut token_pairs_index: Vec<usize> = Vec::new();

            for (i, token) in self.tokens.iter().enumerate() {
                if are_addresses_equal(*token, token_address) {
                    token_found = true;
                    tokens_index = i;

                    for (j, pair) in self.token_pairs.iter().enumerate() {
                        if are_addresses_equal(*token, pair.token0) || are_addresses_equal(*token, pair.token1) {
                            token_pairs_index.push(j);
                        }
                    }
                }
            }

            if token_found {
                self.tokens.remove(tokens_index);

                while !token_pairs_index.is_empty() {
                    self.token_pairs.remove(token_pairs_index.pop().unwrap());
                }
            }
        }

        pub fn tokens(&self) -> &Vec<Address> {
            &self.tokens
        }

        pub fn token_pairs(&self) -> &Vec<TokenPair> {
            &self.token_pairs
        }
    }

    #[tokio::test]
    async fn uniswap_v2_basic_integration_test() -> Result<()> {
        let provider = Provider::try_from("https://eth-mainnet.g.alchemy.com/v2/demo")?;
        let factory_address = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse::<Address>()?;
        let router_address = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse::<Address>()?;
        let mut arbitrage_finder = UniswapV2ArbitrageFinder::new(factory_address, router_address, provider)?;
        assert!(&arbitrage_finder.tokens().is_empty());

        let dai = "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse::<Address>()?;
        let weth = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".parse::<Address>()?;
        let bal = "0xba100000625a3754423978a60c9317c58a424e3D".parse::<Address>()?;
        let non_token = "0x6887246668a3b87F54DeB3b94Ba47a6f63F32985".parse::<Address>()?;

        arbitrage_finder.add_token(dai).await?;
        arbitrage_finder.add_token(weth).await?;
        arbitrage_finder.add_token(bal).await?;
        assert!(arbitrage_finder.tokens().len() == 3_usize);
        assert!(arbitrage_finder.token_pairs().len() == 3_usize);
        arbitrage_finder.add_token(non_token).await?;
        assert!(arbitrage_finder.tokens().len() == 4_usize);
        assert!(arbitrage_finder.token_pairs().len() == 3_usize);
        arbitrage_finder.remove_token(non_token);
        assert!(arbitrage_finder.tokens().len() == 3_usize);
        assert!(arbitrage_finder.token_pairs().len() == 3_usize);
        arbitrage_finder.remove_token(bal);
        assert!(arbitrage_finder.tokens().len() == 2_usize);
        assert!(arbitrage_finder.token_pairs().len() == 1_usize);
        arbitrage_finder.add_token(bal).await?;
        assert!(arbitrage_finder.tokens().len() == 3_usize);
        assert!(arbitrage_finder.token_pairs().len() == 3_usize);

        Ok(())
    }
    // getCurrentPrice
    // Query all price pairs for given tokens, create graph, process in arbitrage engine
}