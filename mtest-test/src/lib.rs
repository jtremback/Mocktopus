#![feature(i128)]
#![feature(i128)]
#![cfg_attr(test, feature(i128))]

extern crate lazy_static;
extern crate rand;
extern crate rand as rand2;

const CONSTANT : u8 = 1 ;

mod another;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_test() {
        assert_eq!(1, CONSTANT);
    }
}
