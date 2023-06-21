use std::fmt;

use crate::term::{Nil, Term};

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct List {
    pub elements: Vec<Term>,
    pub tail: Box<Term>,
}
impl List {
    pub fn iter<'a>(&'a self) -> ListIterator<'a> {
        ListIterator {
            list: self,
            offset: 0,
        }
    }

    /// Returns `true` if it is an improper list (non-nil tail), otherwise `false`.
    pub fn is_improper_list(&self) -> bool {
        if self.tail.is_nil() {
            false
        } else {
            true
        }
    }
}
impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.elements.is_empty() {
            self.tail.fmt(f)
        } else {
            write!(f, "[")?;
            for (i, x) in self.elements.iter().enumerate() {
                if i != 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", x)?;
            }
            if self.is_improper_list() {
                write!(f, "|{}]", self.tail)?;
            } else {
                write!(f, "]")?;
            }
            Ok(())
        }
    }
}
impl From<Vec<Term>> for List {
    fn from(elements: Vec<Term>) -> Self {
        List::from((elements, Term::Nil(Nil)))
    }
}
impl From<(Vec<Term>, Term)> for List {
    fn from((elements, tail): (Vec<Term>, Term)) -> Self {
        List {
            elements,
            tail: Box::new(tail),
        }
    }
}

pub struct ListIterator<'a> {
    list: &'a List,
    offset: usize,
}
impl<'a> ListIterator<'a> {
    pub fn has_next(&self) -> bool {
        self.offset <= self.list.elements.len()
    }
}
impl<'a> Iterator for ListIterator<'a> {
    type Item = &'a Term;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.list.elements.len() {
            None
        } else if self.offset == self.list.elements.len() {
            self.offset += 1;
            Some(&self.list.tail)
        } else {
            let item = &self.list.elements[self.offset];
            self.offset += 1;
            Some(item)
        }
    }
}
