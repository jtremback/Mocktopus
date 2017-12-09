
#![feature(custom_attribute)]

macro_rules! cfg_ls {
    () => { #[cfg(feature = "lazy_static")] }
}


#[cfg_attr(feature = "my_cfg", ym_cfg)]

fn main() {
    #[cfg(feature = "lazy_static")]
    println!("Hello, lazy_static!");
//    cfg_ls!()
    #[cfg(feature = "lazy_static")]
    println!("Hello, cfg_ls!");
    #[cfg(feature = "nothing")]
    println!("Hello, nothing!");
    #[cfg(ym_cfg)]
    println!("Hello, my_cfg!");
    //#[cfg(feature = "mocktopus_injected_run")] //#[test]
    //println!("Hello, mocktopus_injected_run!");
    #[mtest]
    println!("Hello, mtest!");
    println!("Hello, world!");
}
