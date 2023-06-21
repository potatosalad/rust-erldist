use crate::codec::external::ERTS_ATOM_CACHE_SIZE;

use super::{Atom, AtomicOptionRef};

#[derive(Clone, Debug)]
pub struct AtomCache([AtomicOptionRef<Atom>; ERTS_ATOM_CACHE_SIZE]);
impl AtomCache {
    pub fn new() -> Self {
        const NONE: AtomicOptionRef<Atom> = AtomicOptionRef::empty();
        let entries = [NONE; ERTS_ATOM_CACHE_SIZE];
        Self(entries)
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
