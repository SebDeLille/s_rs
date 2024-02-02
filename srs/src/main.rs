use libsrs::add;
use std::env;
use std::rc::Rc;
use libsrs::interpretor::evaluator::Evaluator;
use libsrs::types::core::SrsElement;
use libsrs::types::id::SrsId;
use libsrs::types::integer::SrsInteger;
use libsrs::types::list::{SrsList};


fn main() {

    println!("Hello, world! {}", add(5,6));
    for argument in env::args() {
        println!("{argument}");
    }

    let deux = SrsInteger{value: 2};
    let trois = SrsInteger{value: 3};
    let add = SrsId{value: "+".to_string()};
    let mut list = SrsList::new();
    list.add_tail(Some(Rc::new(add)));
    list.add_tail(Some(Rc::new(deux)));
    list.add_tail(Some(Rc::new(trois)));

}
