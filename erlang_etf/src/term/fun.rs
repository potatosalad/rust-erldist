use std::fmt;

use crate::term::{Atom, Pid, Term};

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub enum Fun {
    InternalFun(InternalFun),
    ExternalFun(ExternalFun),
}
impl fmt::Display for Fun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Fun::InternalFun(ref x) => x.fmt(f),
            Fun::ExternalFun(ref x) => x.fmt(f),
        }
    }
}
impl From<InternalFun> for Fun {
    fn from(x: InternalFun) -> Self {
        Fun::InternalFun(x)
    }
}
impl From<ExternalFun> for Fun {
    fn from(x: ExternalFun) -> Self {
        Fun::ExternalFun(x)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct ExternalFun {
    pub module: Atom,
    pub function: Atom,
    pub arity: u8,
}
impl fmt::Display for ExternalFun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fun {}:{}/{}", self.module, self.function, self.arity)
    }
}
impl<'a, 'b> From<(&'a str, &'b str, u8)> for ExternalFun {
    fn from((module, function, arity): (&'a str, &'b str, u8)) -> Self {
        ExternalFun {
            module: Atom::from(module),
            function: Atom::from(function),
            arity,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub enum InternalFun {
    Old {
        module: Atom,
        pid: Pid,
        free_vars: Vec<Term>,
        index: i32,
        uniq: i32,
    },
    New {
        module: Atom,
        arity: u8,
        pid: Pid,
        free_vars: Vec<Term>,
        index: u32,
        uniq: [u8; 16],
        old_index: i32,
        old_uniq: i32,
    },
}
impl InternalFun {
    pub fn iter<'a>(&'a self) -> InternalFunIterator<'a> {
        let free_vars = match self {
            InternalFun::Old { free_vars, .. } => free_vars,
            InternalFun::New { free_vars, .. } => free_vars,
        };
        InternalFunIterator {
            free_vars,
            offset: 0,
        }
    }
}
impl fmt::Display for InternalFun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InternalFun::Old {
                ref module,
                index,
                uniq,
                ..
            } => write!(f, "#Fun<{}.{}.{}>", module, index, uniq),
            InternalFun::New {
                ref module,
                index,
                uniq,
                ..
            } => {
                let uniq = u128::from_be_bytes(uniq);
                write!(f, "#Fun<{}.{}.{}>", module, index, uniq)
            }
        }
    }
}

pub struct InternalFunIterator<'a> {
    free_vars: &'a Vec<Term>,
    offset: usize,
}
impl<'a> Iterator for InternalFunIterator<'a> {
    type Item = &'a Term;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.free_vars.len() {
            None
        } else {
            let item = &self.free_vars[self.offset];
            self.offset += 1;
            Some(item)
        }
    }
}
