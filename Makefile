lint :; cargo fmt
# Run tests without silencing stdout
test :; cargo test -- --nocapture
# cargo test uniswap_v2_basic_integration_test -- --nocapture --quiet
