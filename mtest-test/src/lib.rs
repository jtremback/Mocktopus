#![feature(i128)]
#![feature(i128)]
#![cfg_attr(test, feature(i128))]

extern crate lazy_static;

const CONSTANT : u8 = 1 ;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_test() {
        assert_eq!(1, CONSTANT);
    }
}
