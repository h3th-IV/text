use drive_car::movement::{drive, reverse}; //use is like import to bring the module into scope

fn check_fuel(){}

pub mod drive_car{
    fn engage_tires(direction: &str) -> Tires{
        let tires: Tires = Tires { front: String::from(direction), rear: String::from("drag") };
        tires
    }

    pub fn drift_tires(direction: &str) -> Tires{
        let tires: Tires = Tires { front: String::from(direction), rear: String::from("drag") };
        tires
    }

    pub mod movement{
        use crate::{check_fuel, drive_car::engage_tires}; //the use statement has to be brought inside the module itself

        pub fn reverse(){
            check_fuel(); //to use item in the parent module we use the super keyword like ../check_fuel
        }
    
        pub fn drive(){
            check_fuel();
        }

        pub fn turn(direction: &str){
            let tire_direction = engage_tires(direction);
            println!("front tires will move like so, {} while rear tires {}", tire_direction.front, tire_direction.rear)
        }
    }

    pub struct Tires{
        pub front: String,
        rear: String
    }
}

fn main(){
    reverse(); //reverse is a public fn in the drive
    drive();

    let drift_dir: drive_car::Tires = drive_car::drift_tires("left_corner");
    println!("a sharp left ahead, drift: {}", drift_dir.front);
    // println!("will the rear tires {}", drift_dir.rear); //error here since rear field is private
}

//sometimes when using the "use" keyword to bring item into scope we can use the "as" keyword to rename it

use std::fmt::Result as fmtResult;

fn func_use_as() -> fmtResult{}

//we can also use "pub" bring to item into scope of whoever is calling our code to ensure the item we brought into our scope is also available to the caller;

pub use std::io::Empty;

//we sometime want to bring multiple item into scope from the same crate
// --snip--
use std::cmp::Ordering;
use std::io;
// --snip--


//we could rewrite the above as 
use std::{cmp::Ordering, io};


//we could also bring two items that share a subpath into scope
use std::io;
use std::io::Write;

//instead we could write it like so
use std::io::{self, Write};

//when we want to bring into scope all public items in a path scope
use std::collections::*;
