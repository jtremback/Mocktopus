# ! [ feature ( proc_macro ) ] # ! [ feature ( custom_attribute ) ] # [ mockable ] extern crate mocktopus as mocktopus_injected_by_mtest ; # [ mockable ] macro_rules ! cfg_ls { ( ) => { # [ cfg ( feature = "lazy_static" ) ] } } # [ cfg_attr ( feature = "my_cfg" , ym_cfg ) ] # [ mockable ] fn main ( ) { # [ cfg ( feature = "lazy_static" ) ] println ! ( "Hello, lazy_static!" ) ; # [ cfg ( feature = "lazy_static" ) ] println ! ( "Hello, cfg_ls!" ) ; # [ cfg ( feature = "nothing" ) ] println ! ( "Hello, nothing!" ) ; # [ cfg ( ym_cfg ) ] println ! ( "Hello, my_cfg!" ) ; # [ mtest ] println ! ( "Hello, mtest!" ) ; println ! ( "Hello, world!" ) ; }