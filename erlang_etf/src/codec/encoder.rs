pub enum TermVisitorHint {
    Continue,
    Skip,
    Halt,
}

pub trait TermVisitor {
    type Return;
    type Error;

    fn visit_outer_term(&mut self, task: &mut Task, term: &Term) -> Result<TermVisitorHint, Self::Error>;
    fn visit_inner_term(&mut self, task: &mut Task, parent: Option<&Term>, child: &Term, hint: InnerTermHint) -> TermVisitorHint;
    fn visit_term_final(&mut self, task: &mut Task, term: &Term) -> TermVisitorHint;
    fn return_final(&mut self, task: &mut Task) -> TaskState<(), Self::Return>;
}

pub trait ReductionCountingTask {
    
}

pub struct ListEncoder {
    task: &mut ReductionCountingTask,
    list: &List,
    iter: ListIterator,
}

pub struct StringEncoder {

}

pub trait Encoder: Task {
    type Ok;
    type Error: std::error::Error;

    type Result: std::result::Result<Self::Ok, Self::Error>;

    fn encode_list(&mut self, list: &List) -> Result;
    fn encode_list_element(&mut self, list: &List, element: &Term) -> Result;
    fn encode_list_tail(&mut self, list: &List, tail: &Term) -> Result;
    fn encode_string(&mut self, list: &List) -> Result;
}