use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::codec::external::ERTS_ATOM_CACHE_SIZE;
use crate::term::Atom;

#[derive(Clone, Debug)]
pub struct AtomRef(Rc<Atom>);
impl AtomRef {
    pub fn new(atom: Atom) -> Self {
        Self(Rc::new(atom))
    }

    pub fn to_owned_atom(&self) -> Atom {
        (*self.0).clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AtomCacheError {
    #[error("{index} is out of atom cache range {range:?}")]
    OutOfRange {
        index: usize,
        range: std::ops::Range<usize>,
    },
}
pub type AtomCacheResult<T> = Result<T, AtomCacheError>;

#[derive(Clone, Debug)]
pub(crate) struct AtomCacheSlots([u128; 16]);
impl AtomCacheSlots {
    pub(crate) fn new() -> Self {
        Self([0; 16])
    }

    #[inline]
    fn get_slot(index: usize) -> (usize, usize) {
        if index < ERTS_ATOM_CACHE_SIZE {
            (index / 128, index % 128)
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get(&self, index: usize) -> bool {
        let (slot, bits) = Self::get_slot(index);
        (self.0[slot] >> bits) & 1 == 1
    }

    pub(crate) fn insert(&mut self, index: usize) -> bool {
        let old_entry = self.get(index);
        let (slot, bits) = Self::get_slot(index);
        self.0[slot] |= 1 << bits;
        old_entry
    }

    pub(crate) fn remove(&mut self, index: usize) -> bool {
        let old_entry = self.get(index);
        let (slot, bits) = Self::get_slot(index);
        self.0[slot] &= !(1 << bits);
        old_entry
    }
}

#[derive(Clone, Debug)]
pub struct AtomCache(Arc<RefCell<AtomCacheInner>>);
impl AtomCache {
    pub fn new() -> Self {
        Self(Arc::new(RefCell::new(AtomCacheInner::new())))
    }

    pub fn get(&self, index: usize) -> AtomCacheResult<Option<AtomRef>> {
        self.0.borrow().get(index).cloned()
    }

    pub fn insert(&self, index: usize, atom: AtomRef) -> AtomCacheResult<Option<AtomRef>> {
        self.0.borrow_mut().insert(index, atom)
    }

    pub fn is_slot_empty(&self, index: usize) -> AtomCacheResult<bool> {
        self.0.borrow().is_slot_empty(index)
    }

    pub fn remove(&mut self, index: usize) -> AtomCacheResult<Option<AtomRef>> {
        self.0.borrow_mut().remove(index)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AtomCacheInner {
    entries: [Option<AtomRef>; ERTS_ATOM_CACHE_SIZE],
    slots: AtomCacheSlots,
}
impl AtomCacheInner {
    pub(crate) fn new() -> Self {
        const NONE: Option<AtomRef> = None;
        Self {
            entries: [NONE; ERTS_ATOM_CACHE_SIZE],
            slots: AtomCacheSlots::new(),
        }
    }

    pub(crate) fn get(&self, index: usize) -> AtomCacheResult<&Option<AtomRef>> {
        if index < ERTS_ATOM_CACHE_SIZE {
            Ok(&self.entries[index])
        } else {
            Err(AtomCacheError::OutOfRange {
                index,
                range: 0..ERTS_ATOM_CACHE_SIZE,
            })
        }
    }

    fn get_mut(&mut self, index: usize) -> AtomCacheResult<&mut Option<AtomRef>> {
        if index < ERTS_ATOM_CACHE_SIZE {
            Ok(&mut self.entries[index])
        } else {
            Err(AtomCacheError::OutOfRange {
                index,
                range: 0..ERTS_ATOM_CACHE_SIZE,
            })
        }
    }

    pub(crate) fn insert(
        &mut self,
        index: usize,
        atom: AtomRef,
    ) -> AtomCacheResult<Option<AtomRef>> {
        let slot = self.get_mut(index)?;
        let old_entry = slot.take();
        *slot = Some(atom);
        self.slots.insert(index);
        Ok(old_entry)
    }

    pub(crate) fn is_slot_empty(&self, index: usize) -> AtomCacheResult<bool> {
        Ok(self.get(index)?.is_none())
    }

    pub(crate) fn remove(&mut self, index: usize) -> AtomCacheResult<Option<AtomRef>> {
        let old_entry = self.get_mut(index)?.take();
        self.slots.remove(index);
        Ok(old_entry)
    }
}

// #[derive(Clone, Debug)]
// pub struct AtomCache(Vec<Option<Atom>>);
// impl AtomCache {
//     pub fn new() -> Self {
//         Self(vec![None; ERTS_ATOM_CACHE_SIZE])
//     }

//     pub fn get(&self, index: usize) -> AtomCacheResult<&Option<Atom>> {
//         if index < self.0.len() {
//             Ok(self.0.get(index).unwrap())
//         } else {
//             Err(AtomCacheError::OutOfRange { index, range: 0..ERTS_ATOM_CACHE_SIZE, })
//         }
//     }

//     pub fn get_mut(&mut self, index: usize) -> AtomCacheResult<&mut Option<Atom>> {
//         if index < self.0.len() {
//             Ok(self.0.get_mut(index).unwrap())
//         } else {
//             Err(AtomCacheError::OutOfRange { index, range: 0..ERTS_ATOM_CACHE_SIZE, })
//         }
//     }

//     pub fn insert(&mut self, index: usize, atom: Atom) -> AtomCacheResult<Option<Atom>> {
//         let slot = self.get_mut(index)?;
//         let old_entry = slot.take();
//         *slot = Some(atom);
//         Ok(old_entry)
//     }

//     pub fn is_slot_empty(&self, index: usize) -> AtomCacheResult<bool> {
//         Ok(self.get(index)?.is_none())
//     }

//     pub fn remove(&mut self, index: usize) -> AtomCacheResult<Option<Atom>> {
//         Ok(self.get_mut(index)?.take())
//     }
// }
