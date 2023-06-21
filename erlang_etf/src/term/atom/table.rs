use std::collections::HashMap;
use std::convert::AsRef;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use super::Atom;

// pub struct Env {
//     pub atom_table: AtomTable,
// }

// impl<'a> From<(&'a Env, &'a str)> for Atom {
//     fn from((env, name): (&'a Env, &'a str)) -> Self {
//         env.atom_table.get_or_insert(&AtomString::from(name))
//     }
// }

#[derive(Debug)]
pub struct AtomTable {
    atoms: Arc<RwLock<HashMap<Atom, usize>>>,
    index: AtomicUsize,
}
impl AtomTable {
    pub fn new() -> Self {
        Self {
            atoms: Arc::new(RwLock::new(HashMap::new())),
            index: AtomicUsize::new(0),
        }
    }

    pub fn get<A: AsRef<Atom>>(&self, atom: A) -> Option<usize> {
        let atoms = self.atoms.read().unwrap();
        let key = atom.as_ref();
        if let Some(val) = atoms.get(key) {
            Some(*val)
        } else {
            None
        }
    }

    pub fn get_or_insert<A: AsRef<Atom>>(&self, atom: A) -> usize {
        if let Some(val) = self.get(&atom) {
            val
        } else {
            let mut atoms = self.atoms.write().unwrap();
            let entry = atoms
                .entry(atom.as_ref().clone())
                .or_insert_with(|| self.index.fetch_add(1, Ordering::Relaxed));
            *entry
        }
    }
}
