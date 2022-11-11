#![allow(dead_code, unused)]

pub use uniswapV2_mod::*;

mod uniswapV2_mod {
    use ethers::{
        providers::{Provider, JsonRpcClient},
        contract::Contract,
        types::{Address, U64, U128, U256},
        core::abi::Abi
    };
    
    use std::{
        fs::File,
        sync::Arc,
        collections::HashMap,
        time::{Duration, Instant}
    };
    
    use eyre::Result;
    use arbitrage_engine::{
        utils::logger::{logText, logObject},
        graph::cycle::get_negative_cycle_quick
    };
    use futures::future::join_all;
    use super::super::utils::{
        is_zero_address, 
        are_addresses_equal,
        one_ether,
        u256_inverse,
        u256_to_f64,
    };
    use petgraph::{
        graph::Graph,
        prelude::{EdgeIndex, NodeIndex},
    };

    #[derive(Debug)]
    struct TokenPair {
        token0: Address,
        token1: Address,
        lptoken: Address
    }

    #[derive(Debug)]
    struct TokenExchangeRates {
        token0: Address,
        token1: Address,
        token0_to_token1_rate: f64,
        token1_to_token0_rate: f64,
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
                        if *token > token_address {
                            self.token_pairs.push(TokenPair{
                                token0: token_address,
                                token1: *token,
                                lptoken: *pair
                            });
                        } else {
                            self.token_pairs.push(TokenPair{
                                token0: *token,
                                token1: token_address,
                                lptoken: *pair
                            });
                        }
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

        pub async fn find_local_arbitrage(&self) -> Result<()> {
            // Build graph with all tokens as nodes
            let mut graph: Graph<Address, f64> = Graph::new();
            let mut nodes: HashMap<Address, NodeIndex> = HashMap::new();
            for i in 0..self.tokens.len() {
                nodes.insert(self.tokens[i], graph.add_node(self.tokens[i]));
            }

            // Get rates for each pair
            let pair_abi_file = File::open("src/abis/UniswapV2Pair.json")?;
            let pair_abi: Abi = serde_json::from_reader(pair_abi_file)?;
            let mut get_reserve_futures = Vec::new();

            for pair_address in self.token_pairs.iter() {
                let pair_contract: Contract<Arc<Provider<P>>> = Contract::new(pair_address.lptoken, pair_abi.clone(), Arc::clone(&self.provider));

                let get_reserve_future = pair_contract
                    .method::<_, (U256, U256, U256)>("getReserves", ())?
                    .call()
                    .await;

                get_reserve_futures.push(tokio::spawn(async move {get_reserve_future}));
            }
            let wrapped_reserves = join_all(get_reserve_futures).await;
            let mut pairs_with_price: Vec<TokenExchangeRates> = Vec::new();
            for (i, pair_struct) in self.token_pairs.iter().enumerate() {
                if wrapped_reserves[i].is_ok() && wrapped_reserves[i].as_ref().unwrap().as_ref().is_ok() {
                    let reserve = wrapped_reserves[i].as_ref().unwrap().as_ref().unwrap();
                    let reserve0 = reserve.0;
                    let reserve1 = reserve.1;

                    // Use UniswapV2Router.getAmountOut() implementation, with 1e18 token0 going in
                    let amount_in_with_fee = one_ether().checked_mul(U256::from_dec_str("997")?).unwrap();
                    let numerator = amount_in_with_fee.checked_mul(reserve1).unwrap();
                    let denominator = reserve0.checked_mul(U256::from_dec_str("1000")?).unwrap().checked_add(amount_in_with_fee).unwrap();
                    let token0_to_token1_rate = numerator.checked_div(denominator).unwrap();
                    let token1_to_token0_price = u256_inverse(token0_to_token1_rate);

                    pairs_with_price.push(TokenExchangeRates {
                        token0: pair_struct.token0,
                        token1: pair_struct.token1,
                        token0_to_token1_rate: u256_to_f64(token0_to_token1_rate),
                        token1_to_token0_rate: u256_to_f64(token1_to_token0_price),
                    });
                }
            }

            // Add rates as nodes to graph
            for pair in pairs_with_price.iter() {
                graph.add_edge(
                    *nodes.get(&pair.token0).unwrap(), 
                    *nodes.get(&pair.token1).unwrap(),
                    pair.token0_to_token1_rate
                );

                graph.add_edge(
                    *nodes.get(&pair.token1).unwrap(), 
                    *nodes.get(&pair.token0).unwrap(),
                    pair.token1_to_token0_rate
                );
            }

            let (is_arbitrage, arbitrage_path) = get_negative_cycle_quick(&graph);

            logObject("is_arbitrage: ", &is_arbitrage);
            logObject("arbitrage_path: ", &arbitrage_path);
            Ok(())
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
        arbitrage_finder.find_local_arbitrage().await?;

        // let rad = "0x31c8eacbffdd875c74b94b077895bd78cf1e64a3".parse::<Address>()?;
        // let paxg = "0x45804880de22913dafe09f4980848ece6ecbaf78".parse::<Address>()?;
        // let uni = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".parse::<Address>()?;
        // let elon = "0x761d38e5ddf6ccf6cf7c55759d5210750b5d60f3".parse::<Address>()?;
        // let fox = "0xc770eefad204b5180df6a14ee197d99d808ee52d".parse::<Address>()?;
        // let wbtc = "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599".parse::<Address>()?;
        // arbitrage_finder.add_token(rad).await?;
        // arbitrage_finder.add_token(paxg).await?;
        // arbitrage_finder.add_token(uni).await?;
        // arbitrage_finder.add_token(elon).await?;
        // arbitrage_finder.add_token(fox).await?;
        // arbitrage_finder.add_token(wbtc).await?;
        // let start = Instant::now();
        // arbitrage_finder.find_local_arbitrage().await?;
        // let duration = start.elapsed();
        // println!("Time elapsed in find_local_arbitrage() is: {:?}", duration);
        Ok(())
    }

    #[tokio::test]
    async fn spookyswap_basic_integration_test() -> Result<()> {
        let provider = Provider::try_from("https://rpc.ftm.tools")?;
        let factory_address = "0x152ee697f2e276fa89e96742e9bb9ab1f2e61be3".parse::<Address>()?;
        let router_address = "0xf491e7b69e4244ad4002bc14e878a34207e38c29".parse::<Address>()?;
        let mut arbitrage_finder = UniswapV2ArbitrageFinder::new(factory_address, router_address, provider)?;
        assert!(&arbitrage_finder.tokens().is_empty());

        let wftm = "0x21be370d5312f44cb42ce377bc9b8a0cef1a4c83".parse::<Address>()?;
        let usdc = "0x04068da6c83afcfa0e13ba15a6696662335d5b75".parse::<Address>()?;
        let spa = "0x5602df4a94eb6c680190accfa2a475621e0ddbdc".parse::<Address>()?;
        let eth = "0x74b23882a30290451a17c44f4f05243b6b58c76d".parse::<Address>()?;
        let btc = "0x321162cd933e2be498cd2267a90534a804051b11".parse::<Address>()?;
        let boo = "0x841fad6eae12c286d1fd18d1d525dffa75c7effe".parse::<Address>()?;
        let midas = "0xb37528da6b4d378305d000a66ad91bd88e626761".parse::<Address>()?;
        let hec = "0x5c4fdfc5233f935f20d2adba572f770c2e377ab0".parse::<Address>()?;
        let link = "0xb3654dc3d10ea7645f8319668e8f54d2574fbdc8".parse::<Address>()?;
        let mimatic = "0xfb98b335551a418cd0737375a2ea0ded62ea213b".parse::<Address>()?;
        let tomb = "0x6c021ae822bea943b2e66552bde1d2696a53fbb7".parse::<Address>()?;
        let tor = "0x74e23df9110aa9ea0b6ff2faee01e740ca1c642e".parse::<Address>()?;
        let mim = "0x82f0b8b456c1a451378467398982d4834b6829c1".parse::<Address>()?;
        let stg = "0x2f6f07cdcf3588944bf4c42ac74ff24bf56e7590".parse::<Address>()?;
        let bnb = "0xd67de0e0a0fd7b15dc8348bb9be742f3c5850454".parse::<Address>()?;
        let geist = "0xd8321aa83fb0a4ecd6348d4577431310a6e0814d".parse::<Address>()?;
        let scream = "0xe0654c8e6fd4d733349ac7e09f6f23da256bf475".parse::<Address>()?;
        let oath = "0x21ada0d2ac28c3a5fa3cd2ee30882da8812279b6".parse::<Address>()?;

        let start = Instant::now();
        arbitrage_finder.add_token(wftm).await?;
        arbitrage_finder.add_token(usdc).await?;
        arbitrage_finder.add_token(spa).await?;
        arbitrage_finder.add_token(eth).await?;
        arbitrage_finder.add_token(btc).await?;
        arbitrage_finder.add_token(boo).await?;
        arbitrage_finder.add_token(midas).await?;
        arbitrage_finder.add_token(hec).await?;
        arbitrage_finder.add_token(link).await?;
        arbitrage_finder.add_token(mimatic).await?;
        arbitrage_finder.add_token(tomb).await?;
        arbitrage_finder.add_token(tor).await?;
        arbitrage_finder.add_token(mim).await?;
        arbitrage_finder.add_token(stg).await?;
        arbitrage_finder.add_token(bnb).await?;
        arbitrage_finder.add_token(geist).await?;
        arbitrage_finder.add_token(scream).await?;
        arbitrage_finder.add_token(oath).await?;
        arbitrage_finder.find_local_arbitrage().await?;
        let duration = start.elapsed();
        println!("Time elapsed in find_local_arbitrage() is: {:?}", duration);
        Ok(())
    }
}