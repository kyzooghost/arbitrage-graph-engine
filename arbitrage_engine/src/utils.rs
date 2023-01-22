#![allow(dead_code, unused, nonstandard_style)]

pub use utils_mod::*;

mod utils_mod {
    pub mod logger {
        use colored::*;
        use std::fmt::Debug;

        pub fn logObject(name: &str, message: &dyn Debug) {
            println!("{}: {:?}", name.blue(), message);
        }

        pub fn logText(test: &str) {
            println!("{}", test.blue());
        }
    }
}