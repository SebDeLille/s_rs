use libsrs::add;
use std::env;
use libsrs::interpretor::evaluator::Evaluator;
use libsrs::types::core::SrsElement;
use libsrs::types::id::SrsId;
use libsrs::types::integer::SrsInteger;
use libsrs::types::list::{SrsList, Iterator};


fn main() {

    println!("Hello, world! {}", add(5,6));
    for argument in env::args() {
        println!("{argument}");
    }

    let deux = SrsInteger{value: 2};
    let trois = SrsInteger{value: 3};
    let numbers: Box<dyn SrsElement> = Box::new(SrsList {
        car: Some(Box::new(deux)),
        cdr: Some(Box::new(SrsList {
            car: Some(Box::new(trois)),
            cdr: None
        }))
    });
    let add = SrsId{value: "+".to_string()};
    let list: Box<dyn SrsElement> = Box::new(SrsList{
        car: Some(Box::new(add)),
        cdr: Some(numbers)
    });

    let tmp = Some(list);
    let mut it = Iterator {
        current_list: tmp.as_ref(),
    };

    if tmp.as_ref().unwrap().is_list() {
        println!("c'est une liste");
    }

    while let Some(tmp) = it.next() {
        let el = tmp.unwrap();
        println!("type: {:?}, {}", el.get_type(), el);

    }

    let e = Evaluator{};
    let _ = e.eval(tmp.as_ref());
}
