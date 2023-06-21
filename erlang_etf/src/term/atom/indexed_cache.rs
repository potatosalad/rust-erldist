use std::convert::AsRef;
use std::fmt;
use std::sync::Arc;

use crate::codec::external::ERTS_ATOM_CACHE_SIZE;

use super::{Atom, AtomicOptionRef, AtomicOptionRefTrait, FromRawPtr, IntoRawPtr};

#[derive(Clone, Debug)]
pub struct IndexedAtom(Arc<IndexedAtomInner>);
impl IndexedAtom {
    pub fn new(iix: u8, atom: &Atom) -> Self {
        Self(Arc::new(IndexedAtomInner {
            atom: atom.clone(),
            iix,
        }))
    }

    pub fn get_atom(&self) -> &Atom {
        &self.0.atom
    }

    pub fn get_iix(&self) -> u8 {
        self.0.iix
    }

    pub fn to_owned_atom(&self) -> Atom {
        self.0.atom.to_owned()
    }
}
impl fmt::Display for IndexedAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.atom.fmt(f)
    }
}
impl AsRef<Atom> for IndexedAtom {
    fn as_ref(&self) -> &Atom {
        &self.0.atom
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
impl AtomicOptionRefTrait for IndexedAtom {}

#[derive(Clone, Debug)]
struct IndexedAtomInner {
    atom: Atom,
    iix: u8,
}

#[derive(Clone, Debug)]
pub struct IndexedAtomCache([AtomicOptionRef<IndexedAtom>; ERTS_ATOM_CACHE_SIZE]);
impl IndexedAtomCache {
    pub fn new() -> Self {
        const NONE: AtomicOptionRef<IndexedAtom> = AtomicOptionRef::empty();
        let entries = [NONE; ERTS_ATOM_CACHE_SIZE];
        Self(entries)
    }

    pub fn get(&self, index: usize) -> Option<IndexedAtom> {
        self.0[index].load()
    }

    pub fn insert(&self, index: usize, indexed_atom: IndexedAtom) -> Option<IndexedAtom> {
        self.0[index].swap(Some(indexed_atom))
    }

    pub fn insert_if_empty(&self, index: usize, indexed_atom: IndexedAtom) -> bool {
        self.0[index].store_if_none(indexed_atom)
    }

    pub fn is_slot_empty(&self, index: usize) -> bool {
        self.get(index).is_none()
    }

    pub fn remove(&self, index: usize) -> Option<IndexedAtom> {
        self.0[index].swap(None)
    }
}
