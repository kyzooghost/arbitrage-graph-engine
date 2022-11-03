pub use utils_mod::*;

mod utils_mod {
    pub mod logger {
        use colored::*;
        use std::fmt::Debug;

        pub fn log(name: &str, message: &dyn Debug) {
            println!("{}: {:?}", name.blue(), message);
        }
    }
}