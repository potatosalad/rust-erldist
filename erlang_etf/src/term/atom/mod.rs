use std::fmt;
use std::sync::Arc;

mod atomic_option_ref;
mod cache;
mod indexed_cache;
pub mod table;

pub use atomic_option_ref::{AtomicOptionRef, AtomicOptionRefTrait, FromRawPtr, IntoRawPtr};
pub use cache::*;
pub(crate) use indexed_cache::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct AtomString {
    pub name: String,
}
impl fmt::Display for AtomString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "'{}'",
            self.name.replace('\\', "\\\\").replace('\'', "\\'")
        )
    }
}
impl<'a> From<&'a str> for AtomString {
    fn from(name: &'a str) -> Self {
        AtomString {
            name: name.to_string(),
        }
    }
}
impl From<String> for AtomString {
    fn from(name: String) -> Self {
        AtomString { name }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Atom(Arc<AtomString>);
impl Atom {
    pub fn len(&self) -> usize {
        self.0.name.len()
    }
}
// impl Atom {
//     fn into_raw_atom_string(this: Self) -> *const AtomString {
//         Arc::into_raw(this.0)
//     }

//     unsafe fn from_raw_atom_string(ptr: *const AtomString) -> Self {
//         Atom(unsafe { Arc::from_raw(ptr) })
//     }
// }
// impl From<AtomString> for Atom {
//     fn from(raw_atom: AtomString) -> Atom {
//         Atom(Arc::new(raw_atom))
//     }
// }
impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<T: Into<AtomString>> From<T> for Atom {
    fn from(input: T) -> Atom {
        Atom(Arc::new(input.into()))
    }
}
impl<'a> AsRef<Atom> for &'a Atom {
    fn as_ref(&self) -> &Atom {
        self
    }
}
impl FromRawPtr for Atom {
    unsafe fn from_raw_ptr(ptr: *mut ()) -> Self {
        Atom(unsafe { Arc::from_raw(ptr as *const AtomString) })
    }
}
impl IntoRawPtr for Atom {
    fn into_raw_ptr(self) -> *mut () {
        Arc::into_raw(self.0) as *mut ()
        // Atom::into_raw_atom_string(self) as *mut ()
    }
}
impl AtomicOptionRefTrait for Atom {}

// pub trait IntoAtom {
//     fn into_atom(self) -> Atom;
// }
// impl IntoAtom for AtomString {
//     fn into_atom(self) -> Atom {
//         Atom(Arc::new(self))
//     }
// }
// impl IntoAtom for Atom {
//     fn into_atom(self) -> Atom {
//         self
//     }
// }
// impl IntoAtom for &Atom {
//     fn into_atom(self) -> Atom {
//         self.clone()
//     }
// }

// #[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
// pub struct Atom {
//     /// The name of the atom.
//     pub name: String,
// }
// impl Atom {
//     pub fn len(&self) -> usize {
//         self.name.len()
//     }
// }
// impl fmt::Display for Atom {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "'{}'",
//             self.name.replace('\\', "\\\\").replace('\'', "\\'")
//         )
//     }
// }
// impl<'a> From<&'a str> for Atom {
//     fn from(name: &'a str) -> Self {
//         Atom {
//             name: name.to_string(),
//         }
//     }
// }
// impl From<String> for Atom {
//     fn from(name: String) -> Self {
//         Atom { name }
//     }
// }
