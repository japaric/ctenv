#![no_std]

#[macro_export]
macro_rules! ctenv {
    ($key:ident) => {
        include!(concat!(env!("OUT_DIR"), "/", stringify!($key)))
    }
}
