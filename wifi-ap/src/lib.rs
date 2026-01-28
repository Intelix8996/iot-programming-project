#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![recursion_limit = "1024"]

pub mod wifi_ap;
pub mod wifi_sta;
pub mod web;
pub mod uart;
pub mod dhcp;

#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
