#[cfg_attr(kani, kani::ensures(|result| *result == x))]
pub fn identity(x: u8) -> u8 { x }

#[cfg(kani)]
#[kani::proof_for_contract(identity)]
fn check_identity() {
    let x: u8 = ::kani::any();
    identity(x);
}
