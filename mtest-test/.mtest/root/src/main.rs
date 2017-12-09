#![feature(proc_macro)]
#[mockable]
extern crate mocktopus as mocktopus_injected_by_mtest;

#[mockable]
fn main() { println!("Hello, world!"); }
