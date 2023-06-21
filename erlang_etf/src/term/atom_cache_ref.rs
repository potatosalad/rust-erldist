// use std::convert::AsRef;
use std::fmt;
use std::ops::Index;
// use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use crate::codec::external::{ERTS_MAX_INTERNAL_ATOM_CACHE_ENTRIES, ERTS_USE_ATOM_CACHE_SIZE};

use super::atom::{Atom, IndexedAtom, IndexedAtomCache};

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct AtomCacheRef {
    pub index: u8,
}
impl AtomCacheRef {
    pub fn new(index: u8) -> Self {
        Self { index }
    }
}
impl fmt::Display for AtomCacheRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#AtomCacheRef<{}>", self.index)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub enum AtomCacheRefEntry {
    New { index: usize, atom_text: Vec<u8> },
    Old { index: usize },
}
impl AtomCacheRefEntry {
    pub fn new(index: usize, atom_text: Vec<u8>) -> Self {
        Self::New { index, atom_text }
    }

    pub fn old(index: usize) -> Self {
        Self::Old { index }
    }
}

// #[derive(Debug)]
// pub struct AtomCacheIndex(AtomicUsize);
// impl Clone for AtomCacheIndex {
//     fn clone(&self) -> Self {
//         Self(AtomicUsize::new(self.0.load(Ordering::Relaxed)))
//     }
// }

#[derive(Clone, Debug)]
pub struct AtomCacheRefBuilder(Arc<RwLock<AtomCacheRefBuilderInner>>);
impl AtomCacheRefBuilder {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(AtomCacheRefBuilderInner::new())))
    }

    pub fn get(&self, atom: &Atom) -> Option<AtomCacheRef> {
        self.0.read().unwrap().get(atom)
    }

    pub fn insert(&self, atom: &Atom) -> Option<AtomCacheRef> {
        self.0.write().unwrap().insert(atom)
    }
}

#[derive(Debug)]
pub(crate) struct AtomCacheRefBuilderInner {
    sz: usize,
    long_atoms: bool,
    cix: [usize; ERTS_MAX_INTERNAL_ATOM_CACHE_ENTRIES],
    cache: IndexedAtomCache,
}
impl AtomCacheRefBuilderInner {
    pub(crate) fn new() -> Self {
        Self {
            sz: 0,
            long_atoms: false,
            cix: [usize::MAX; ERTS_MAX_INTERNAL_ATOM_CACHE_ENTRIES],
            cache: IndexedAtomCache::new(),
        }
    }

    pub(crate) fn get(&self, atom: &Atom) -> Option<AtomCacheRef> {
        let ix = Self::atom2cix(atom);
        if let Some(entry) = self.cache.get(ix) {
            if atom.eq(entry.get_atom()) {
                return Some(AtomCacheRef::new(entry.get_iix()));
            }
        }
        None
    }

    pub fn insert(&mut self, atom: &Atom) -> Option<AtomCacheRef> {
        if self.sz < ERTS_MAX_INTERNAL_ATOM_CACHE_ENTRIES {
            let iix = self.sz;
            let ix = Self::atom2cix(atom);
            let indexed_atom = IndexedAtom::new(iix as u8, atom);
            if self.cache.insert_if_empty(ix, indexed_atom) {
                if !self.long_atoms && atom.len() > 255 {
                    self.long_atoms = true;
                }
                self.cix[iix] = ix;
                self.sz += 1;
                return Some(AtomCacheRef::new(iix as u8));
            }
        }
        None
    }

    fn atom2cix(atom: &Atom) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        atom.hash(&mut hasher);
        (hasher.finish() as usize) % ERTS_USE_ATOM_CACHE_SIZE
    }
}
// impl From<IndexedAtomRefCache> for AtomCacheIndexer {
//     fn from(cache: IndexedAtomRefCache) -> Self {
//         let mut map = AtomCacheIndexer::new();
//         map.cache = cache;
//         map
//     }
// }
