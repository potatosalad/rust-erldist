use std::fmt;

use crate::term::Atom;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Pid {
    pub node: Atom,
    pub id: u32,
    pub serial: u32,
    pub creation: u32,
}
impl Pid {
    pub fn new<T>(node: T, id: u32, serial: u32, creation: u32) -> Self
    where
        Atom: From<T>,
    {
        Pid {
            node: Atom::from(node),
            id,
            serial,
            creation,
        }
    }
}
impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}.{}.{}>", self.node, self.id, self.serial)
    }
}
