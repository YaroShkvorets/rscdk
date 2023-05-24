#![cfg_attr(not(feature = "std"), no_std)]

use rust_chain as chain;
mod mymod;

#[chain::contract]
mod hello {
    use super::mymod;
    use rust_chain::{
        Asset,
        Name,
        eosio_print,
        chain_println,
        name,
    };

    use mymod::hello::{
        BB
    };
    ///
    #[cfg_attr(feature = "std", derive(rust_chain::eosio_scale_info::TypeInfo))]
    #[cfg_attr(feature = "std", scale_info(crate = ::rust_chain::eosio_scale_info))]
    #[derive(Clone, Eq, PartialEq, Default)]
    pub struct MyChecksum {
        data: [u8; 20],
        data2: mymod::hello::BB,
    }

    struct AA {
        value: u64,
    }

    pub enum MyVariant1 {
        Var1(u64)
    }

    #[chain(variant)]
    pub enum MyVariant2 {
        Var2(u64)
    }

    pub enum MyVariant3 {
        Var2(u64, u64)
    }

    pub struct MyData3 {
        count: u64,
        asset: Asset,
        myvariant: MyVariant1,
    }

    pub struct MyData2 {
        count: u64,
        mydata: MyData3,
    }

    #[chain(table="mydata")]
    pub struct MyData {
        #[chain(primary)]
        a1: u64,
        #[chain(secondary)]
        a2: u64,
        mydata: MyData2,
        aa1: AA,
        bb: BB,
    }

    #[chain(main)]
    pub struct Hello {
        receiver: Name,
        first_receiver: Name,
        action: Name,
    }

    impl Hello {

        pub fn new(receiver: Name, first_receiver: Name, action: Name) -> Self {
            Self {
                receiver: receiver,
                first_receiver: first_receiver,
                action: action,
            }
        }

        #[chain(action="test")]
        pub fn test(&self, name: String) {
            let mut v = vec![1, 2, 3, 4];
            chain_println!("++++hello:", name, v);
        }
    }
}
