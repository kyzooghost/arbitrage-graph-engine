pub use utils_mod::*;

mod utils_mod {
    use std::ops::Div;

    use ethers::types::{Address, U256};
    
    pub fn is_zero_address(address: Address) -> bool {
        address.to_fixed_bytes() == "0x0000000000000000000000000000000000000000".parse::<Address>().unwrap().to_fixed_bytes()
    }

    pub fn are_addresses_equal(address1: Address, address2: Address) -> bool {
        address1.to_fixed_bytes() == address2.to_fixed_bytes()
    }

    pub fn one_ether() -> U256 {
        U256::exp10(18)
    }

    pub fn u256_inverse(x: U256) -> U256 {
        U256::exp10(36).checked_div(x).unwrap()
    }

    pub fn u256_to_f64(x: U256) -> f64 {
        let (integer, mantissa) = x.div_mod(one_ether());
        // So this will error if our integer larger than 1e18?
        let integer_as_f64 = integer.as_u64() as f64;
        // Mantissa is % 1e18, so cannot be bigger than 1e18 by definition
        // u64 as safely handle numbers up to ~1e19
        let mantissa_as_f64 = mantissa.as_u64() as f64 / 10_u64.pow(18) as f64;
        integer_as_f64 + mantissa_as_f64
    }
}
