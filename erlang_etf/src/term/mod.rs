use std::fmt;

pub mod convert;

mod atom;
mod atom_cache_ref;
mod bitstring;
mod dist;
mod fun;
mod list;
mod map;
mod nil;
mod number;
mod pid;
mod port;
mod reference;
mod tuple;

pub use atom::*;
pub use atom_cache_ref::*;
pub use bitstring::*;
pub use dist::*;
pub use fun::*;
pub use list::*;
pub use map::*;
pub use nil::*;
pub use number::*;
pub use pid::*;
pub use port::*;
pub use reference::*;
pub use tuple::*;

/// See [Term Comparisons](https://www.erlang.org/doc/reference_manual/expressions.html#term-comparisons) in the Erlang docs.
/// Ordering: `number < atom < reference < fun < port < pid < tuple < map < nil < list < bit string`
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub enum Term {
    Number(Number),
    Atom(Atom),
    Reference(Reference),
    Fun(Fun),
    Port(Port),
    Pid(Pid),
    Tuple(Tuple),
    Map(Map),
    Nil(Nil),
    List(List),
    Bitstring(Bitstring),
    Dist(Dist),
}
impl Term {
    /// Returns `true` if it is fun term, otherwise `false`.
    pub fn is_fun(&self) -> bool {
        match *self {
            Term::Fun(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if it is internal fun term, otherwise `false`.
    pub fn is_internal_fun(&self) -> bool {
        match *self {
            Term::Fun(Fun::InternalFun(_)) => true,
            _ => false,
        }
    }

    /// Returns `true` if it is internal fun term, otherwise `false`.
    pub fn is_external_fun(&self) -> bool {
        match *self {
            Term::Fun(Fun::ExternalFun(_)) => true,
            _ => false,
        }
    }

    /// Returns `true` if it is list term, otherwise `false`.
    pub fn is_list(&self) -> bool {
        match *self {
            Term::Nil(_) => true,
            Term::List(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if it is map term, otherwise `false`.
    pub fn is_map(&self) -> bool {
        match *self {
            Term::Map(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if it is nil term, otherwise `false`.
    pub fn is_nil(&self) -> bool {
        match *self {
            Term::Nil(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if it is tuple term, otherwise `false`.
    pub fn is_tuple(&self) -> bool {
        match *self {
            Term::Tuple(_) => true,
            _ => false,
        }
    }
}
impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Term::Number(ref x) => x.fmt(f),
            Term::Atom(ref x) => x.fmt(f),
            Term::Reference(ref x) => x.fmt(f),
            Term::Fun(ref x) => x.fmt(f),
            Term::Port(ref x) => x.fmt(f),
            Term::Pid(ref x) => x.fmt(f),
            Term::Tuple(ref x) => x.fmt(f),
            Term::Map(ref x) => x.fmt(f),
            Term::Nil(ref x) => x.fmt(f),
            Term::List(ref x) => x.fmt(f),
            Term::Bitstring(ref x) => x.fmt(f),
            Term::Dist(ref x) => x.fmt(f),
        }
    }
}
impl From<Atom> for Term {
    fn from(x: Atom) -> Self {
        Term::Atom(x)
    }
}
impl From<Bitstring> for Term {
    fn from(x: Bitstring) -> Self {
        Term::Bitstring(x)
    }
}
impl From<Dist> for Term {
    fn from(x: Dist) -> Self {
        Term::Dist(x)
    }
}
impl From<Fun> for Term {
    fn from(x: Fun) -> Self {
        Term::Fun(x)
    }
}
impl From<List> for Term {
    fn from(x: List) -> Self {
        Term::List(x)
    }
}
impl From<Map> for Term {
    fn from(x: Map) -> Self {
        Term::Map(x)
    }
}
impl From<Nil> for Term {
    fn from(x: Nil) -> Self {
        Term::Nil(x)
    }
}
impl From<Number> for Term {
    fn from(x: Number) -> Self {
        Term::Number(x)
    }
}
impl From<Pid> for Term {
    fn from(x: Pid) -> Self {
        Term::Pid(x)
    }
}
impl From<Port> for Term {
    fn from(x: Port) -> Self {
        Term::Port(x)
    }
}
impl From<Reference> for Term {
    fn from(x: Reference) -> Self {
        Term::Reference(x)
    }
}
impl From<Tuple> for Term {
    fn from(x: Tuple) -> Self {
        Term::Tuple(x)
    }
}
