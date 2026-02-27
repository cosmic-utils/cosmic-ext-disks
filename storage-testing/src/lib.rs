extern crate self as storage_testing;

pub mod artifacts;
pub mod cmd;
pub mod errors;
pub mod harness;
pub mod lab;
pub mod ledger;
pub mod runtime;
pub mod spec;
#[path = "../tests/mod.rs"]
pub mod tests;

#[cfg(test)]
mod self_tests {
    #[test]
    fn crate_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
