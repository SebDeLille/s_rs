use crate::types::core::{SrsElement, SrsType, SrsValue, SrsValueRef};
use crate::types::error::{SrsError, SrsErrorKind, SrsResult};
use crate::types::id::SrsId;
use crate::types::integer::SrsInteger;
use crate::types::list::SrsList;
use crate::types::memory::SrsMemory;

pub struct Evaluator<'a> {
    memory: Box<SrsMemory<'a>>,
}

impl Evaluator<'_> {
    pub fn new() -> Self {
        Evaluator { memory: Box::new(SrsMemory::new()) }
    }

    pub fn eval(&self, element: Option<&Box<dyn SrsElement>>) -> SrsResult<SrsValue> {
        match element.unwrap().get_type() {
            SrsType::LIST => self.eval_list(element),
            SrsType::INTEGER => self.eval_integer(element),
            SrsType::ID => self.eval_id(element),
            _ => Err(SrsError { kind: SrsErrorKind::UnknownType })
        }
    }

    fn eval_list(&self, list: SrsValueRef) -> SrsResult<SrsValue> {
        match list.unwrap().as_any().downcast_ref::<SrsList>() {
            Some(_) => Ok(None),
            None => Err(SrsError { kind: SrsErrorKind::DowncastFail })
        }
    }

    fn eval_integer(&self, i: SrsValueRef) -> SrsResult<SrsValue> {
        // match i.unwrap().as_any().downcast_ref::<SrsInteger>() {
        //     Some(int) => Ok(Some(Box::new(SrsInteger { value: int.value }))),
        //     None => Err(SrsError { kind: SrsErrorKind::DowncastFail })
        // }
        Ok(None)
    }

    fn eval_id(&self, id: SrsValueRef) -> SrsResult<SrsValue> {
        match id.unwrap().as_any().downcast_ref::<SrsId>() {
            Some(id) => Ok(None),
            None => Err(SrsError { kind: SrsErrorKind::DowncastFail })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpretor::evaluator::Evaluator;
    use crate::interpretor::lexical_analyzer::get_lexemes;
    use crate::interpretor::translator::translate_all;
    use crate::to_type;
    use crate::types::core::SrsType;
    use crate::types::integer::SrsInteger;

    #[test]
    fn basic() {
        let evaluator = Evaluator::new();
        let scm = "2";
        let tmp_res = get_lexemes(&scm.to_string());
        let result = translate_all(tmp_res.unwrap());
        // match result {
        //     Ok(v) => {
        //         let value = evaluator.eval(v.get(0));
        //         match value {
        //             Ok(res) => {
        //                 assert_eq!(res.as_ref().unwrap().get_type(), SrsType::INTEGER);
        //                 let i = to_type!(res, SrsInteger).unwrap();
        //                 assert_eq!(i.value, 2);
        //             }
        //             Err(e) => panic!("{}", e)
        //         }
        //     }
        //     Err(e) => panic!("{}", e)
        // }
    }
}