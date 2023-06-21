// mod decoder;
mod dist;
mod external;

// use std::ops::DerefMut;
// use std::pin::Pin;

use crate::task::{Task, TaskHint, TaskState};
use crate::term::{Fun, InternalFunIterator, ListIterator, MapIterator, Term, TupleIterator};

pub enum InnerTermHint {
    InternalFunFreeVar,
    ListElement,
    ListTail,
    MapKey,
    MapValue,
    OuterTerm,
    TupleElement,
}

enum EncoderTaskState<'a> {
    OuterTerm(&'a Term),
    InnerTerm { parent: Option<&'a Term>, child: &'a Term, hint: InnerTermHint, },
    InternalFunIterator { term: &'a Term, iter: InternalFunIterator<'a> },
    ListIterator { term: &'a Term, iter: ListIterator<'a> },
    MapIterator { term: &'a Term, iter: MapIterator<'a> },
    TupleIterator { term: &'a Term, iter: TupleIterator<'a> },
}

pub enum TermVisitorHint {
    Continue,
    Skip,
    Halt,
}

pub trait TermVisitor {
    type Return;

    fn visit_outer_term(&mut self, task: &mut Task, term: &Term) -> TermVisitorHint;
    fn visit_inner_term(&mut self, task: &mut Task, parent: Option<&Term>, child: &Term, hint: InnerTermHint) -> TermVisitorHint;
    fn visit_term_final(&mut self, task: &mut Task, term: &Term) -> TermVisitorHint;
    fn return_final(&mut self, task: &mut Task) -> TaskState<(), Self::Return>;
}

pub struct EncoderTask<'a, T: TermVisitor> {
    visitor: T,
    task: Task,
    stack: Vec<EncoderTaskState<'a>>,
}
impl<'a, T: TermVisitor> EncoderTask<'a, T> {
    pub fn new(visitor: T, task: Task, term: &'a Term) -> Self {
        EncoderTask { task, stack: vec![EncoderTaskState::OuterTerm(term)], visitor }
    }

    pub fn resume(&mut self) -> TaskState<(), T::Return> {
        self.task.reset_after_yield();
        while let Some(state) = self.stack.pop() {
            match state {
                EncoderTaskState::OuterTerm(term) => {
                    // println!("[outer] term = {:?}", term);
                    match self.visitor.visit_outer_term(&mut self.task, term) {
                        TermVisitorHint::Continue => {
                            self.stack.push(EncoderTaskState::InnerTerm { parent: None, child: term, hint: InnerTermHint::OuterTerm });
                        }
                        TermVisitorHint::Skip => (),
                        TermVisitorHint::Halt => self.stack.clear(),
                    };
                }
                EncoderTaskState::InnerTerm { parent, child, hint } => {
                    // println!("[inner] term = {:?}", child);
                    match self.visitor.visit_inner_term(&mut self.task, parent, child, hint) {
                        TermVisitorHint::Continue => {
                            match child {
                                Term::Number(_) => todo!(),
                                Term::Atom(_) => {
                                    // println!("[inner] atom = {}", x);
                                }
                                Term::Reference(_) => todo!(),
                                Term::Fun(x) => {
                                    // println!("[inner] fun = {}", x);
                                    match x {
                                        Fun::ExternalFun(_) => todo!(),
                                        Fun::InternalFun(i) => {
                                            self.stack.push(EncoderTaskState::InternalFunIterator { term: child, iter: i.iter() });
                                        }
                                    };
                                }
                                Term::Port(_) => todo!(),
                                Term::Pid(_) => todo!(),
                                Term::Tuple(x) => {
                                    // println!("[inner] tuple = {}", x);
                                    self.stack.push(EncoderTaskState::TupleIterator { term: child, iter: x.iter() });
                                }
                                Term::Map(x) => {
                                    // println!("[inner] map = {}", x);
                                    self.stack.push(EncoderTaskState::MapIterator { term: child, iter: x.iter() });
                                }
                                Term::Nil(x) => {
                                    // println!("[inner] nil = {}", x);
                                }
                                Term::List(x) => {
                                    // println!("[inner] list = {}", x);
                                    self.stack.push(EncoderTaskState::ListIterator { term: child, iter: x.iter() });
                                }
                                Term::Bitstring(_) => todo!(),
                            };
                        }
                        TermVisitorHint::Skip => (),
                        TermVisitorHint::Halt => self.stack.clear(),
                    };
                }
                EncoderTaskState::InternalFunIterator { term, mut iter } => {
                    if let Some(free_var) = iter.next() {
                        self.stack.push(EncoderTaskState::InternalFunIterator { term, iter });
                        let hint = InnerTermHint::InternalFunFreeVar;
                        self.stack.push(EncoderTaskState::InnerTerm { parent: Some(term), child: free_var, hint });
                    } else {
                        match self.visitor.visit_term_final(&mut self.task, term) {
                            TermVisitorHint::Continue => (),
                            TermVisitorHint::Skip => (),
                            TermVisitorHint::Halt => self.stack.clear(),
                        };
                    }
                }
                EncoderTaskState::ListIterator { term, mut iter } => {
                    if let Some(element) = iter.next() {
                        let hint = if iter.has_next() { InnerTermHint::ListElement } else { InnerTermHint::ListTail };
                        self.stack.push(EncoderTaskState::ListIterator { term, iter });
                        self.stack.push(EncoderTaskState::InnerTerm { parent: Some(term), child: element, hint });
                    } else {
                        match self.visitor.visit_term_final(&mut self.task, term) {
                            TermVisitorHint::Continue => (),
                            TermVisitorHint::Skip => (),
                            TermVisitorHint::Halt => self.stack.clear(),
                        };
                    }
                }
                EncoderTaskState::MapIterator { term, mut iter } => {
                    if let Some((key, value)) = iter.next() {
                        self.stack.push(EncoderTaskState::MapIterator { term, iter });
                        self.stack.push(EncoderTaskState::InnerTerm { parent: Some(term), child: value, hint: InnerTermHint::MapValue });
                        self.stack.push(EncoderTaskState::InnerTerm { parent: Some(term), child: key, hint: InnerTermHint::MapKey });
                    } else {
                        match self.visitor.visit_term_final(&mut self.task, term) {
                            TermVisitorHint::Continue => (),
                            TermVisitorHint::Skip => (),
                            TermVisitorHint::Halt => self.stack.clear(),
                        };
                    }
                }
                EncoderTaskState::TupleIterator { term, mut iter } => {
                    if let Some(element) = iter.next() {
                        let hint = InnerTermHint::TupleElement;
                        self.stack.push(EncoderTaskState::TupleIterator { term, iter });
                        self.stack.push(EncoderTaskState::InnerTerm { parent: Some(term), child: element, hint });
                    } else {
                        match self.visitor.visit_term_final(&mut self.task, term) {
                            TermVisitorHint::Continue => (),
                            TermVisitorHint::Skip => (),
                            TermVisitorHint::Halt => self.stack.clear(),
                        };
                    }
                }
            }
            if let TaskHint::Yield = self.task.bump_reds(1) {
                return TaskState::Yielded(());
            }
        };
        self.visitor.return_final(&mut self.task)
    }
}

// pub struct SizeEncoder {

// }
// impl SizeEncoder {
//     pub fn encode(term: &Term) -> usize {
//         let mut task = EncoderTask::new(SizeEncoderVisitor::new())
//     }
// }

pub struct SizeEncoder {
    byte_size: usize,
}
impl TermVisitor for SizeEncoder {
    type Return = usize;

    fn visit_outer_term(&mut self, task: &mut Task, term: &Term) -> TermVisitorHint {
        println!("[visit_outer_term] {}", term);
        TermVisitorHint::Continue
    }

    fn visit_inner_term(&mut self, task: &mut Task, parent: Option<&Term>, child: &Term, hint: InnerTermHint) -> TermVisitorHint {
        println!("[visit_inner_term] {}", child);
        TermVisitorHint::Continue
    }

    fn visit_term_final(&mut self, task: &mut Task, term: &Term) -> TermVisitorHint {
        println!("[visit_term_final] {}", term);
        TermVisitorHint::Continue
    }

    fn return_final(&mut self, task: &mut Task) -> TaskState<(), Self::Return> {
        TaskState::Complete(1)
    }
}
impl SizeEncoder {
    pub fn encode(term: &Term) -> usize {
        let visitor = Self { byte_size: 0 };
        let mut encoder_task = EncoderTask::new(visitor, Task::blocking(), term);
        // let mut encoder_task = EncoderTask::new(visitor, Task::yielding(7), term);
        loop {
            match encoder_task.resume() {
                TaskState::Yielded(()) => {
                    println!("yielded after {:?} reductions", encoder_task.task.get_consumed_reds());
                    continue;
                }
                TaskState::Complete(result) => {
                    println!("remaining {:?} reductions", encoder_task.task.get_remaining_reds());
                    return result;
                }
            }
        }
    }
}

// pub struct Encoder<'a> {
//     task: Task,
//     size: usize,
//     stack: Vec<EncoderTaskState<'a>>,
// }
// impl<'a> Encoder<'a> {
//     pub fn encode(term: &'a Term) -> usize {
//         let mut encoder = Self::generate_task(Task::blocking(), term);
//         // let mut encoder = Self::generate_task(Task::yielding(7), term);
//         loop {
//             match Self::resume_task(Pin::new(&mut encoder)) {
//                 TaskState::Yielded(()) => {
//                     println!("yielded after {:?} reductions", encoder.get_consumed_reds());
//                     continue;
//                 }
//                 TaskState::Complete(result) => {
//                     println!("remaining {:?} reductions", encoder.get_remaining_reds());
//                     return result;
//                 },
//             }
//         }
//     }

//     pub fn generate_task(task: Task, term: &'a Term) -> Self {
//         Encoder { task, size: 0, stack: vec![EncoderTaskState::OuterTerm(term)] }
//     }

//     pub fn get_consumed_reds(&self) -> usize {
//         self.task.get_consumed_reds()
//     }

//     pub fn get_remaining_reds(&self) -> usize {
//         self.task.get_remaining_reds()
//     }

//     pub fn resume_task(mut self: Pin<&mut Self>) -> TaskState<(), usize> {
//         self.task.reset_after_yield();
//         while let Some(state) = self.stack.pop() {
//             match state {
//                 EncoderTaskState::OuterTerm(term) => {
//                     println!("[outer] term = {:?}", term);
//                     self.stack.push(EncoderTaskState::InnerTerm(term));
//                 }
//                 EncoderTaskState::InnerTerm(term) => {
//                     println!("[inner] term = {:?}", term);
//                     match term {
//                         Term::Number(_) => todo!(),
//                         Term::Atom(x) => {
//                             println!("[inner] atom = {}", x);
//                         }
//                         Term::Reference(_) => todo!(),
//                         Term::Fun(x) => {
//                             println!("[inner] fun = {}", x);
//                             match x {
//                                 Fun::ExternalFun(_) => todo!(),
//                                 Fun::InternalFun(i) => {
//                                     self.stack.push(EncoderTaskState::InternalFunIterator(i.iter()));
//                                 }
//                             };
//                         }
//                         Term::Port(_) => todo!(),
//                         Term::Pid(_) => todo!(),
//                         Term::Tuple(x) => {
//                             println!("[inner] tuple = {}", x);
//                             self.stack.push(EncoderTaskState::TupleIterator(x.iter()));
//                         }
//                         Term::Map(x) => {
//                             println!("[inner] map = {}", x);
//                             self.stack.push(EncoderTaskState::MapIterator(x.iter()));
//                         }
//                         Term::Nil(x) => {
//                             println!("[inner] nil = {}", x);
//                         }
//                         Term::List(x) => {
//                             println!("[inner] list = {}", x);
//                             self.stack.push(EncoderTaskState::ListIterator(x.iter()));
//                         }
//                         Term::Bitstring(_) => todo!(),
//                     };
//                 }
//                 EncoderTaskState::InternalFunIterator(mut iter) => {
//                     if let Some(free_var) = iter.next() {
//                         self.stack.push(EncoderTaskState::InternalFunIterator(iter));
//                         self.stack.push(EncoderTaskState::InnerTerm(free_var));
//                     }
//                 }
//                 EncoderTaskState::ListIterator(mut iter) => {
//                     if let Some(element) = iter.next() {
//                         self.stack.push(EncoderTaskState::ListIterator(iter));
//                         self.stack.push(EncoderTaskState::InnerTerm(element));
//                     }
//                 }
//                 EncoderTaskState::MapIterator(mut iter) => {
//                     if let Some((key, value)) = iter.next() {
//                         self.stack.push(EncoderTaskState::MapIterator(iter));
//                         self.stack.push(EncoderTaskState::InnerTerm(value));
//                         self.stack.push(EncoderTaskState::InnerTerm(key));
//                     }
//                 }
//                 EncoderTaskState::TupleIterator(mut iter) => {
//                     if let Some(element) = iter.next() {
//                         self.stack.push(EncoderTaskState::TupleIterator(iter));
//                         self.stack.push(EncoderTaskState::InnerTerm(element));
//                     }
//                 }
//             }
//             if let TaskHint::Yield = self.task.bump_reds(1) {
//                 return TaskState::Yielded(());
//             }
//         }
//         TaskState::Complete(self.size)
//         // TaskState::Yielded(())
//     }

//     // pub fn encode(mut self, term: Term) -> Option<usize> {
//     //     match *term {
//     //         Term::Atom(ref x) => Ok(1),
//     //         _ => None,
//     //     }
//     // }
// }
