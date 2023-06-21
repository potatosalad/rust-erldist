use std::fmt;

use super::AtomCacheRefEntry;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub enum Dist {
    DistHeader(DistHeader),
    // DistFragHeader(DistFragHeader),
    // DistFragCont(DistFragCont),
}
impl fmt::Display for Dist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Dist::DistHeader(ref x) => x.fmt(f),
        }
    }
}
impl From<DistHeader> for Dist {
    fn from(x: DistHeader) -> Self {
        Dist::DistHeader(x)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct DistHeader {
    pub long_atoms: bool,
    pub atom_cache_ref_entries: Vec<AtomCacheRefEntry>,
}
impl fmt::Display for DistHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "#DistHeader<long_atoms={:?}, atom_cache_ref_entries=[",
            self.long_atoms
        )?;
        for (i, entry) in self.atom_cache_ref_entries.iter().enumerate() {
            match entry {
                AtomCacheRefEntry::New { index, atom_text } => {
                    write!(f, "({:?}, {:?})", index, atom_text)?;
                }
                AtomCacheRefEntry::Old { index } => {
                    write!(f, "{:?}", index)?;
                }
            };
            if i != self.atom_cache_ref_entries.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]>")
    }
}
