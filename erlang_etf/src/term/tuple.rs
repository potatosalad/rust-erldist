use std::fmt;

use crate::term::Term;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Tuple {
    pub elements: Vec<Term>,
}
impl Tuple {
    pub fn empty() -> Self {
        Tuple {
            elements: Vec::new(),
        }
    }

    pub fn iter<'a>(&'a self) -> TupleIterator<'a> {
        TupleIterator {
            tuple: self,
            offset: 0,
        }
    }
}
impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for (i, x) in self.elements.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", x)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
impl From<Vec<Term>> for Tuple {
    fn from(elements: Vec<Term>) -> Self {
        Tuple { elements }
    }
}

pub struct TupleIterator<'a> {
    tuple: &'a Tuple,
    offset: usize,
}
impl<'a> Iterator for TupleIterator<'a> {
    type Item = &'a Term;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.tuple.elements.len() {
            None
        } else {
            let item = &self.tuple.elements[self.offset];
            self.offset += 1;
            Some(item)
        }
    }
}
