use creusot_std::prelude::*;
#[ensures(result == x)]
pub fn identity(x: u8) -> u8 { x }
