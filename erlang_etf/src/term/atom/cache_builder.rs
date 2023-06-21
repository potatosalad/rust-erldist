use std::convert::AsRef;
use std::fmt;
use std::sync::Arc;

use crate::codec::external::{
    ERTS_ATOM_CACHE_SIZE,
};

use super::{Atom, AtomicOptionRef, AtomicOptionRefTrait, FromRawPtr, IntoRawPtr};

#[derive(Clone, Debug)]
pub struct IndexedAtom(Arc<IndexedAtomInner>);
impl IndexedAtom {
    pub fn new<I: Into<Atom>>(iix: u8, atom: I) -> Self {
        Self(Arc::new(IndexedAtomInner {
            atom: atom.into(),
            iix,
        }))
    }

    pub fn to_owned_atom(&self) -> Atom {
        self.atom_ref.as_ref().to_owned()
    }
}
impl fmt::Display for IndexedAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.atom_ref.fmt(f)
    }
}
impl AsRef<Atom> for IndexedAtom {
    fn as_ref(&self) -> &Atom {
        self.0.atom.as_ref()
    }
}
impl From<IndexedAtom> for Atom {
    fn from(source: IndexedAtom) -> Atom {
        source.0.atom.clone()
    }
}
impl FromRawPtr for IndexedAtom {
    unsafe fn from_raw_ptr(ptr: *mut ()) -> Self {
        IndexedAtom(unsafe { Arc::from_raw(ptr as *const IndexedAtomInner) })
    }
}
impl IntoRawPtr for IndexedAtom {
    fn into_raw_ptr(self) -> *mut () {
        Arc::into_raw(self.0) as *mut ()
    }
}
impl AtomicOptionRefTrait for Atom {}

#[derive(Clone, Debug)]
struct IndexedAtomInner {
    atom: Atom,
    iix: u8,
}

#[derive(Clone, Debug)]
pub struct IndexedAtomCache(Arc<[AtomicOptionRef<IndexedAtom>; ERTS_ATOM_CACHE_SIZE]>);
impl IndexedAtomCache {
    pub fn new() -> Self {
        const NONE: AtomicOptionRef<IndexedAtom> = AtomicOptionRef::default();
        let entries = [NONE; ERTS_ATOM_CACHE_SIZE];
        Self(Arc::new(entries))
    }

    pub fn get(&self, index: usize) -> Option<Atom> {
        self.0[index].load()
    }

    pub fn insert(&mut self, index: usize, atom: Atom) -> Option<Atom> {
        self.0[index].swap(Some(atom))
    }

    pub fn is_slot_empty(&self, index: usize) -> bool {
        self.get(index).is_none()
    }

    pub fn remove(&mut self, index: usize) -> Option<Atom> {
        self.0[index].swap(None)
    }
}
