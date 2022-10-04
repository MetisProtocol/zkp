// #[macro_use]
// extern crate pest_derive;
// extern crate pest;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nom;
extern crate nom_locate;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate arrayref;

extern crate itertools;


pub mod utils;

pub mod mem;
pub mod mips_lang;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
