pub use utils_mod::*;

mod utils_mod {
    use ethers::types::Address;
    
    pub fn is_zero_address(address: Address) -> bool {
        address.to_fixed_bytes() == "0x0000000000000000000000000000000000000000".parse::<Address>().unwrap().to_fixed_bytes()
    }

    pub fn are_addresses_equal(address1: Address, address2: Address) -> bool {
        address1.to_fixed_bytes() == address2.to_fixed_bytes()
    }
}
