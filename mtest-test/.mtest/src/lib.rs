#![feature(proc_macro)]
#![feature(i128)]
#![feature(i128)]
#![cfg_attr(test, feature(i128))]
extern crate mocktopus as mocktopus_injected_by_mtest;
extern crate lazy_static;
extern crate rand;
extern crate rand as rand2;

const CONSTANT: u8 = 1;

#[mockable]
mod another;

#[cfg(test)]
#[mockable]
mod tests {
    use super::*;

    #[test]
    fn constant_test() { assert_eq!(1, CONSTANT); }
}
