use std::fmt;

use crate::term::Atom;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Reference {
    pub node: Atom,
    pub id: Vec<u32>,
    pub creation: u32,
}
impl Reference {
    pub fn new<T>(node: T, id: Vec<u32>, creation: u32) -> Self
    where
        Atom: From<T>,
    {
        Reference {
            node: Atom::from(node),
            id,
            creation,
        }
    }
}
impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#Ref<{}", self.node)?;
        for n in &self.id {
            write!(f, ".{}", n)?;
        }
        write!(f, ">")
    }
}
// impl<'a> From<(&'a str, u32)> for Reference {
//     fn from((node, id): (&'a str, u32)) -> Self {
//         Reference {
//             node: Atom::from(node),
//             id: vec![id],
//             creation: 0,
//         }
//     }
// }
// impl<'a> From<(&'a str, Vec<u32>)> for Reference {
//     fn from((node, id): (&'a str, Vec<u32>)) -> Self {
//         Reference {
//             node: Atom::from(node),
//             id,
//             creation: 0,
//         }
//     }
// }
