use std::fmt;

use crate::term::Term;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Map {
    pub pairs: Vec<(Term, Term)>,
}
impl Map {
    pub fn empty() -> Self {
        Map { pairs: Vec::new() }
    }

    pub fn iter<'a>(&'a self) -> MapIterator<'a> {
        MapIterator {
            map: self,
            offset: 0,
        }
    }
}
impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{{")?;
        for (i, &(ref k, ref v)) in self.pairs.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            write!(f, "{}=>{}", k, v)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
impl From<Vec<(Term, Term)>> for Map {
    fn from(pairs: Vec<(Term, Term)>) -> Self {
        Map { pairs }
    }
}

pub struct MapIterator<'a> {
    map: &'a Map,
    offset: usize,
}
impl<'a> Iterator for MapIterator<'a> {
    type Item = &'a (Term, Term);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.map.pairs.len() {
            None
        } else {
            let item = &self.map.pairs[self.offset];
            self.offset += 1;
            Some(item)
        }
    }
}
