#![feature(i128)]
#![feature(proc_macro)]
#![cfg_attr(test, feature(i128))]

extern crate lazy_static;
extern crate rand;
extern crate rand as rand2;
#[cfg(test)]
extern crate mocktopus;
#[cfg(test)]
use mocktopus::macros::mocktopus_test;

const CONSTANT : u8 = 1 ;

mod another;

#[cfg(test)]
mod tests {

    extern crate rand as rand3;
    use super::*;

    #[test]
    fn constant_test() {
        rand::thread_rng();
        rand2::thread_rng();
        assert_eq!(1, CONSTANT);
    }

    #[mocktopus_test]
    fn sometimes_test() {
        rand::thread_rng();
        rand2::thread_rng();
        assert_eq!(1, CONSTANT);
    }
}
