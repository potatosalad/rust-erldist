use std::fmt;

use crate::term::Atom;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Port {
    pub node: Atom,
    pub id: u64,
    pub creation: u32,
}
impl Port {
    pub fn new<T>(node: T, id: u64, creation: u32) -> Self
    where
        Atom: From<T>,
    {
        Port {
            node: Atom::from(node),
            id,
            creation,
        }
    }
}
impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#Port<{}.{}>", self.node, self.id)
    }
}
// impl<'a> From<(&'a str, u32)> for Port {
//     fn from((node, id): (&'a str, u32)) -> Self {
//         Port {
//             node: Atom::from(node),
//             id: u64::from(id),
//             creation: 0,
//         }
//     }
// }
// impl<'a> From<(&'a str, u64)> for Port {
//     fn from((node, id): (&'a str, u64)) -> Self {
//         Port {
//             node: Atom::from(node),
//             id,
//             creation: 0,
//         }
//     }
// }
