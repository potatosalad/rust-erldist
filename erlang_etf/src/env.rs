use std::cmp::{Ordering, PartialOrd};
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::mem::ManuallyDrop;
use std::borrow::{Borrow, Cow};
use std::collections::hash_map::{DefaultHasher, HashMap, RandomState};
use std::ops::Deref;
use std::sync::{Arc, Weak};

use bstr::{BStr, ByteSlice};
use parking_lot::RwLock;

use crate::bytes::{BytesMut, Bytes};

#[derive(Clone, Debug)]
pub struct Env {
    atom_table: Arc<RwLock<AtomTable>>,
}
impl Env {
    pub fn new() -> Self {
        Self {
            atom_table: Arc::new(RwLock::new(AtomTable::new())),
        }
    }

    pub fn make_atom<T: Into<Cow<'static, [u8]>>>(&self, atom_name: T) -> Atom {
        self.make_atom_utf8(atom_name)
    }

    pub fn make_atom_latin1<T: Into<Cow<'static, [u8]>>>(&self, atom_name: T) -> Atom {
        self.make_atom_int(AtomEncoding::Latin1, atom_name)
    }

    pub fn make_atom_utf8<T: Into<Cow<'static, [u8]>>>(&self, atom_name: T) -> Atom {
        self.make_atom_int(AtomEncoding::Utf8, atom_name)
    }

    fn make_atom_int<T: Into<Cow<'static, [u8]>>>(&self, atom_encoding: AtomEncoding, atom_name: T) -> Atom {
        let atom_name = atom_name.into();
        if let Some(atom) = self.atom_table.read().raw_get(atom_encoding, &atom_name) {
            return atom.clone();
        }
        self.atom_table.write().raw_get_or_intern(atom_encoding, atom_name)
    }
}

/// Default capacity for a new [`AtomTable`] created with
/// [`AtomTable::new`].
pub const DEFAULT_ATOM_TABLE_CAPACITY: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct AtomHashValue(u32);
impl From<u32> for AtomHashValue {
    fn from(x: u32) -> Self {
        Self(x)
    }
}
impl From<AtomHashValue> for u32 {
    fn from(x: AtomHashValue) -> Self {
        x.0
    }
}
impl From<AtomEntry> for AtomHashValue {
    fn from(x: AtomEntry) -> Self {
        Self::from(x.hash_value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct AtomSlotIndex(usize);
impl From<usize> for AtomSlotIndex {
    fn from(x: usize) -> Self {
        Self(x)
    }
}
impl From<AtomSlotIndex> for usize {
    fn from(x: AtomSlotIndex) -> Self {
        x.0
    }
}
impl From<AtomEntry> for AtomSlotIndex {
    fn from(x: AtomEntry) -> Self {
        Self::from(x.slot_index)
    }
}

#[derive(Eq)]
struct AtomKey<'a> {
    encoded_size: usize,
    encoding: AtomEncoding,
    bytes: &'a [u8],
}
impl<'a> AtomKey<'a> {
    fn new(encoding: AtomEncoding, bytes: &'a [u8]) -> Self {
        Self {
            encoded_size: if encoding.is_latin1() { AtomEntry::atom_latin1_to_utf8(bytes).count() } else { bytes.len() },
            encoding,
            bytes,
        }
    }

    fn iter_bytes(&self) -> AtomEntryBytesIterator<'a> {
        if self.needs_latin1_encoding() {
            AtomEntryBytesIterator::Latin1(AtomEntry::atom_latin1_to_utf8(self.bytes))
        } else {
            AtomEntryBytesIterator::Utf8(self.bytes.iter())
        }
    }

    fn needs_latin1_encoding(&self) -> bool {
        self.encoded_size > self.bytes.len()
    }
}
impl<'a> fmt::Debug for AtomKey<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // f.debug_tuple("AtomKey").field()
        write!(f, "AtomKey('{}')", self.bytes.as_bstr())
            //.replace('\\', "\\\\").replace('\'', "\\'")
        // )
        // f.debug_struct("AtomKey").field("encoded_size", &self.encoded_size).field("encoding", &self.encoding).field("bytes", &self.bytes).finish()
    }
}
impl<'a> Hash for AtomKey<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(AtomEntry::atom_hashpjw(self.bytes));
    }
}
impl<'a> PartialEq for AtomKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        if !self.encoded_size.eq(&other.encoded_size) {
            return false;
        }
        if self.needs_latin1_encoding() || other.needs_latin1_encoding() {
            return self.iter_bytes().eq(other.iter_bytes());
        }
        self.bytes.eq(other.bytes)
    }
}
impl<'a> PartialOrd for AtomKey<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.encoded_size.partial_cmp(&other.encoded_size) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        if self.needs_latin1_encoding() || other.needs_latin1_encoding() {
            return self.iter_bytes().partial_cmp(other.iter_bytes());
        }
        self.bytes.partial_cmp(other.bytes)
    }
}

// pub enum AtomEntryKey {
//     Latin1(&'static [u8]),
//     Utf8(),
// }

#[derive(Default, Debug)]
pub struct AtomTable<S = RandomState> {
    map: ManuallyDrop<HashMap<AtomKey<'static>, Atom, S>>,
    vec: ManuallyDrop<Vec<Arc<AtomEntry>>>,
}
impl<S> Drop for AtomTable<S> {
    fn drop(&mut self) {
        // Safety:
        //
        // `Interned` requires that the `'static` references it gives out are
        // dropped before the owning buffer stored in the `Interned`.
        //
        // `ManuallyDrop::drop` is only invoked in this `Drop::drop` impl;
        // because mutable references to `map` and `vec` fields are not given
        // out by `AtomTable`, `map` and `vec` are guaranteed to be
        // initialized.
        unsafe {
            ManuallyDrop::drop(&mut self.map);
            ManuallyDrop::drop(&mut self.vec);
        }
    }
}
impl AtomTable<RandomState> {
    /// Constructs a new, empty `AtomTable` with [default capacity].
    ///
    /// This function will always allocate. To construct a symbol table without
    /// allocating, call [`AtomTable::with_capacity(0)`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use intaglio::bytes::AtomTable;
    /// let table = AtomTable::new();
    /// assert_eq!(0, table.len());
    /// assert!(table.capacity() >= 4096);
    /// ```
    ///
    /// [default capacity]: DEFAULT_SYMBOL_TABLE_CAPACITY
    /// [`AtomTable::with_capacity(0)`]: Self::with_capacity
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_ATOM_TABLE_CAPACITY)
    }

    /// Constructs a new, empty `AtomTable` with the specified capacity.
    ///
    /// The symbol table will be able to hold at least `capacity` byte strings
    /// without reallocating. If `capacity` is 0, the symbol table will not
    /// allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// # use intaglio::bytes::AtomTable;
    /// let table = AtomTable::with_capacity(10);
    /// assert_eq!(0, table.len());
    /// assert!(table.capacity() >= 10);
    /// ```
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        Self {
            map: ManuallyDrop::new(HashMap::with_capacity(capacity)),
            vec: ManuallyDrop::new(Vec::with_capacity(capacity)),
        }
    }
}

impl<S> AtomTable<S> {
    /// Constructs a new, empty `AtomTable` with
    /// [default capacity](DEFAULT_SYMBOL_TABLE_CAPACITY) and the given hash
    /// builder.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::hash_map::RandomState;
    /// # use intaglio::bytes::AtomTable;
    /// let hash_builder = RandomState::new();
    /// let table = AtomTable::with_hasher(hash_builder);
    /// assert_eq!(0, table.len());
    /// assert!(table.capacity() >= 4096);
    /// ```
    pub fn with_hasher(hash_builder: S) -> Self {
        Self::with_capacity_and_hasher(DEFAULT_ATOM_TABLE_CAPACITY, hash_builder)
    }

    /// Constructs a new, empty `AtomTable` with the specified capacity and
    /// the given hash builder.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::hash_map::RandomState;
    /// # use intaglio::bytes::AtomTable;
    /// let hash_builder = RandomState::new();
    /// let table = AtomTable::with_capacity_and_hasher(10, hash_builder);
    /// assert_eq!(0, table.len());
    /// assert!(table.capacity() >= 10);
    /// ```
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            map: ManuallyDrop::new(HashMap::with_capacity_and_hasher(capacity, hash_builder)),
            vec: ManuallyDrop::new(Vec::with_capacity(capacity)),
        }
    }

    /// Returns the number of byte strings the table can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// # use intaglio::bytes::AtomTable;
    /// let table = AtomTable::with_capacity(10);
    /// assert!(table.capacity() >= 10);
    /// ```
    pub fn capacity(&self) -> usize {
        usize::min(self.vec.capacity(), self.map.capacity())
    }

    /// Returns the number of interned byte strings in the table.
    ///
    /// # Examples
    ///
    /// ```
    /// # use intaglio::bytes::AtomTable;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut table = AtomTable::new();
    /// assert_eq!(0, table.len());
    ///
    /// table.intern(b"abc".to_vec())?;
    /// // only uniquely interned byte strings grow the symbol table.
    /// table.intern(b"abc".to_vec())?;
    /// table.intern(b"xyz".to_vec())?;
    /// assert_eq!(2, table.len());
    /// # Ok(())
    /// # }
    /// # example().unwrap();
    /// ```
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns `true` if the symbol table contains no interned byte strings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use intaglio::bytes::AtomTable;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut table = AtomTable::new();
    /// assert!(table.is_empty());
    ///
    /// table.intern(b"abc".to_vec())?;
    /// assert!(!table.is_empty());
    /// # Ok(())
    /// # }
    /// # example().unwrap();
    /// ```
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Returns `true` if the symbol table contains the given symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// # use intaglio::bytes::AtomTable;
    /// # use intaglio::Symbol;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut table = AtomTable::new();
    /// assert!(!table.contains(Symbol::new(0)));
    ///
    /// let sym = table.intern(b"abc".to_vec())?;
    /// assert!(table.contains(Symbol::new(0)));
    /// assert!(table.contains(sym));
    /// # Ok(())
    /// # }
    /// # example().unwrap();
    /// ```
    #[must_use]
    pub fn contains(&self, id: Atom) -> bool {
        self.get(id).is_some()
    }

    /// Returns a reference to the byte string associated with the given symbol.
    ///
    /// If the given symbol does not exist in the underlying symbol table,
    /// `None` is returned.
    ///
    /// The lifetime of the returned reference is bound to the symbol table.
    ///
    /// # Examples
    ///
    /// ```
    /// # use intaglio::bytes::AtomTable;
    /// # use intaglio::Symbol;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut table = AtomTable::new();
    /// assert!(table.get(Symbol::new(0)).is_none());
    ///
    /// let sym = table.intern(b"abc".to_vec())?;
    /// assert_eq!(Some(&b"abc"[..]), table.get(Symbol::new(0)));
    /// assert_eq!(Some(&b"abc"[..]), table.get(sym));
    /// # Ok(())
    /// # }
    /// # example().unwrap();
    /// ```
    #[must_use]
    pub fn get(&self, atom: Atom) -> Option<&[u8]> {
        let entry = self.vec.get(usize::from(atom.slot_index()))?;
        Some(entry.name())
    }

    #[must_use]
    pub fn get_entry(&self, atom: Atom) -> Option<&AtomEntry> {
        let entry = self.vec.get(usize::from(atom.slot_index()))?;
        Some(entry)
    }
}

impl<S> AtomTable<S>
where
    S: BuildHasher,
{
    fn raw_get<'a>(&'a self, atom_encoding: AtomEncoding, atom_name: &'a Cow<'static, [u8]>) -> Option<&'a Atom> {
        let atom_read_key = AtomKey::new(atom_encoding, &*atom_name);
        self.map.get(&atom_read_key)
    }

    fn raw_get_or_intern(&mut self, atom_encoding: AtomEncoding, atom_name: Cow<'static, [u8]>) -> Atom {
        let atom_write_key_encoded_size = {
            let atom_read_key = AtomKey::new(atom_encoding, &*atom_name);
            if let Some(atom) = self.map.get(&atom_read_key) {
                return atom.clone();
            }
            atom_read_key.encoded_size
        };
        let atom_name = Interned::from(atom_name);
        let atom_slot_index: AtomSlotIndex = self.vec.len().into();
        // Safety:
        //
        // This expression creates a reference with a `'static` lifetime
        // from an owned and interned buffer. This is permissible because:
        //
        // - `Interned` is an internal implementation detail of `AtomTable`.
        // - `AtomTable` never give out `'static` references to underlying
        //   byte contents.
        // - All slice references given out by the `AtomTable` have the same
        //   lifetime as the `AtomTable`.
        // - The `map` field of `AtomTable`, which contains the `'static`
        //   references, is dropped before the owned buffers stored in this
        //   `Interned`.
        let slice = unsafe { atom_name.as_static_slice() };
        let atom_write_key = AtomKey { encoded_size: atom_write_key_encoded_size, encoding: atom_encoding, bytes: slice, };
        let atom_entry = Arc::new(AtomEntry::from_intern(atom_slot_index, atom_write_key_encoded_size, atom_encoding, atom_name));
        let atom = Atom::from(Arc::clone(&atom_entry));

        self.map.insert(atom_write_key, atom.clone());
        self.vec.push(atom_entry);

        atom
    }

    /// Intern a byte string for the lifetime of the symbol table.
    ///
    /// The returned `Symbol` allows retrieving of the underlying bytes.
    /// Equal byte strings will be inserted into the symbol table exactly once.
    ///
    /// This function only allocates if the underlying symbol table has no
    /// remaining capacity.
    ///
    /// # Errors
    ///
    /// If the symbol table would grow larger than `u32::MAX` interned
    /// byte strings, the [`Symbol`] counter would overflow and a
    /// [`SymbolOverflowError`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use intaglio::bytes::AtomTable;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut table = AtomTable::new();
    /// let sym = table.intern(b"abc".to_vec())?;
    /// table.intern(b"xyz".to_vec())?;
    /// table.intern(&b"123"[..])?;
    /// table.intern(&b"789"[..])?;
    ///
    /// assert_eq!(4, table.len());
    /// assert_eq!(Some(&b"abc"[..]), table.get(sym));
    /// # Ok(())
    /// # }
    /// # example().unwrap();
    /// ```
    // pub fn intern<T>(&mut self, contents: T) -> Result<Atom, SymbolOverflowError>
    pub fn intern<T>(&mut self, atom_encoding: AtomEncoding, atom_name: T) -> Atom
    where
        T: Into<Cow<'static, [u8]>>,
    {
        let atom_name = atom_name.into();
        self.raw_get_or_intern(atom_encoding, atom_name)
        // let contents = contents.into();
        // let write_key_encoded_size = {
        //     let read_key = AtomKey::new(encoding, &*contents);
        //     if let Some(atom) = self.map.get(&read_key) {
        //         return atom.clone();
        //     }
        //     read_key.encoded_size
        // };
        // let name = Interned::from(contents);
        // let atom_slot_index: AtomSlotIndex = self.vec.len().into();
        // // Safety:
        // //
        // // This expression creates a reference with a `'static` lifetime
        // // from an owned and interned buffer. This is permissible because:
        // //
        // // - `Interned` is an internal implementation detail of `AtomTable`.
        // // - `AtomTable` never give out `'static` references to underlying
        // //   byte contents.
        // // - All slice references given out by the `AtomTable` have the same
        // //   lifetime as the `AtomTable`.
        // // - The `map` field of `AtomTable`, which contains the `'static`
        // //   references, is dropped before the owned buffers stored in this
        // //   `Interned`.
        // let slice = unsafe { name.as_static_slice() };
        // let write_key = AtomKey { encoded_size: write_key_encoded_size, encoding, bytes: slice, };
        // let atom_entry = Arc::new(AtomEntry::from_intern(atom_slot_index, write_key_encoded_size, encoding, name));
        // let atom = Atom::from(Arc::clone(&atom_entry));

        // self.map.insert(write_key, atom.clone());
        // self.vec.push(atom_entry);

        // // debug_assert_eq!(self.get(atom), Some(&atom_entry));
        // // debug_assert_eq!(self.intern(slice), Ok(id));

        // atom
    }

    pub fn intern_latin1<T>(&mut self, contents: T) -> Atom
    where
        T: Into<Cow<'static, [u8]>>,
    {
        self.intern(AtomEncoding::Latin1, contents)
    }

    pub fn intern_utf8<T>(&mut self, contents: T) -> Atom
    where
        T: Into<Cow<'static, [u8]>>,
    {
        self.intern(AtomEncoding::Utf8, contents)
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd)]
#[repr(transparent)]
struct AtomLatin1Name(Vec<u8>);
impl fmt::Debug for AtomLatin1Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "'{:?}'", self.0.replace(b"\\", b"\\\\").replace(b"\'", "\\'").as_bstr())
        let atom_name = format!("{:?}", self.0.as_bstr());
        write!(f, "'{}'", &atom_name[1..atom_name.len() - 1])
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Atom(Arc<AtomEntry>);
impl From<Arc<AtomEntry>> for Atom {
    fn from(x: Arc<AtomEntry>) -> Self {
        Self(x)
    }
}
impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.needs_latin1_encoding() {
            return f.debug_tuple("Atom").field(&AtomLatin1Name(self.iter_bytes().collect())).finish();
        }
        let atom_name = format!("{:?}", self.name().as_bstr());
        write!(f, "Atom('{}')", &atom_name[1..atom_name.len() - 1])
        // write!(f, "Atom('{}')", self.name().as_bstr().replace(b"\\\\", b"\\\\\\\\").replace(b"\\'", "\\\\'").as_bstr())
        // write!(
        //     f,
        //     "'{}'",
        //     self.bytes.as_bstr()//.replace('\\', "\\\\").replace('\'', "\\'")
        // )
        // f.debug_struct("AtomKey").field("encoded_size", &self.encoded_size).field("encoding", &self.encoding).field("bytes", &self.bytes).finish()
    }
}
impl Deref for Atom {
    type Target = Arc<AtomEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
// impl Atom {
//     pub fn new(slot_index: AtomSlotIndex, atom_entry: Weak<AtomEntry>) -> Self {
//         Self { slot_index, atom_entry, }
//     }

//     pub fn as_bytes<'a>(&'a self) -> Option<&'a [u8]> {
//         if let Some(atom_entry) = self.atom_entry() {
//             return Some(atom_entry.name());
//         }
//         None
//     }

//     pub fn atom_entry(&self) -> Option<Arc<AtomEntry>> {
//         self.atom_entry.upgrade()
//     }

//     pub fn slot_index(&self) -> AtomSlotIndex {
//         self.slot_index
//     }
// }
// impl From<Atom> for usize {
//     fn from(atom: Atom) -> Self {
//         atom.0
//     }
// }
// impl From<usize> for Atom {
//     fn from(index: usize) -> Self {
//         Self(index)
//     }
// }

#[derive(Debug, Eq)]
pub struct AtomEntry {
    hash_value: AtomHashValue,
    slot_index: AtomSlotIndex,
    ord0: i32,
    encoded_size: usize,
    encoding: AtomEncoding,
    name: Interned<[u8]>,
}
impl AtomEntry {
    pub fn encoding(&self) -> AtomEncoding {
        self.encoding
    }

    pub fn hash_value(&self) -> AtomHashValue {
        self.hash_value
    }

    pub fn iter_bytes<'a>(&'a self) -> AtomEntryBytesIterator<'a> {
        if self.needs_latin1_encoding() {
            AtomEntryBytesIterator::Latin1(Self::atom_latin1_to_utf8(self.name.as_slice()))
        } else {
            AtomEntryBytesIterator::Utf8(self.name.as_slice().iter())
        }
    }

    pub fn name<'a>(&'a self) -> &'a [u8] {
        self.name.as_slice()
    }

    pub fn needs_latin1_encoding(&self) -> bool {
        self.encoded_size > self.name.len()
    }

    pub fn slot_index(&self) -> AtomSlotIndex {
        self.slot_index
    }

    fn from_intern(slot_index: AtomSlotIndex, encoded_size: usize, encoding: AtomEncoding, name: Interned<[u8]>) -> Self {
        Self {
            hash_value: AtomHashValue::from(Self::atom_hashpjw(name.as_slice())),
            slot_index,
            ord0: Self::atom_ord0(encoded_size, name.as_slice()),
            encoded_size,
            encoding,
            name,
        }
    }

    fn atom_hashpjw(text: &[u8]) -> u32 {
        let mut h: u32 = 0;
        for v in text {
            h = (h << 4) + (*v as u32);
            let g = h & 0xf0000000;
            if g > 0 {
                h ^= g >> 24;
                h ^= g;
            }
        }
        h
    }

    fn atom_latin1_to_utf8<'a>(text: &'a [u8]) -> AtomEntryBytesLatin1ToUtf8Iterator<'a> {
        AtomEntryBytesLatin1ToUtf8Iterator {
            offset: 0,
            byte: None,
            text,
        }
    }

    fn atom_ord0(encoded_size: usize, text: &[u8]) -> i32 {
        let mut c: [i32; 4] = [0; 4];
        if encoded_size > text.len() {
            for (i, chr) in Self::atom_latin1_to_utf8(text).enumerate().take(4) {
                c[i] = chr as i32;
            }
        } else {
            for i in 0..usize::min(4, text.len()) {
                c[i] = text[i] as i32;
            }
        }
        (c[0] << 23) + (c[1] << 15) + (c[2] << 7) + (c[3] >> 1)
    }
}
impl PartialEq for AtomEntry {
    fn eq(&self, other: &Self) -> bool {
        if !self.ord0.eq(&other.ord0) {
            return false;
        }
        if self.needs_latin1_encoding() || other.needs_latin1_encoding() {
            return self.iter_bytes().eq(other.iter_bytes());
        }
        self.name.eq(&other.name)
    }
}
impl PartialOrd for AtomEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.ord0.partial_cmp(&other.ord0) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        };
        if self.needs_latin1_encoding() || other.needs_latin1_encoding() {
            return self.iter_bytes().partial_cmp(other.iter_bytes());
        }
        self.name.as_slice().partial_cmp(other.name.as_slice())
    }
}

#[derive(Clone, Debug)]
pub enum AtomEntryBytesIterator<'a> {
    Latin1(AtomEntryBytesLatin1ToUtf8Iterator<'a>),
    Utf8(std::slice::Iter<'a, u8>),
}
impl<'a> Iterator for AtomEntryBytesIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Latin1(inner) => inner.next(),
            Self::Utf8(inner) => inner.next().copied(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AtomEntryBytesLatin1ToUtf8Iterator<'a> {
    offset: usize,
    byte: Option<u8>,
    text: &'a [u8],
}
impl<'a> Iterator for AtomEntryBytesLatin1ToUtf8Iterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.text.len() {
            return None;
        }
        if let Some(chr) = self.byte.take() {
            self.offset += 1;
            return Some(chr);
        }
        let chr = self.text[self.offset];
        if chr & 0x80 == 0 {
            self.offset += 1;
            return Some(chr);
        }
        let fst = 0xC0 | (chr >> 6);
        let snd = 0x80 | (chr & 0x3F);
        self.byte = Some(snd);
        return Some(fst);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub enum AtomEncoding {
    Latin1,
    Utf8,
}
impl AtomEncoding {
    pub fn is_latin1(&self) -> bool {
        match self {
            AtomEncoding::Latin1 => true,
            _ => false,
        }
    }

    pub fn is_utf8(&self) -> bool {
        match self {
            AtomEncoding::Utf8 => true,
            _ => false,
        }
    }
}
impl Default for AtomEncoding {
    fn default() -> Self {
        Self::Utf8
    }
}

// impl Hash for AtomEntry {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         let mut h: u32 = 0;
//         let mut g: u32 = 0;
//         let mut len = self.name.len();
//         let mut i: usize = 0;
//         while len > 0 {
//             len -= 1;
//             let mut v = self.name[i] as u32;
//             i += 1;
//             // latin1 clutch for r16
//             if len > 0 && (v & 0xFE) == 0xC2 && (self.name[i] & 0xC0) == 0x80 {
//                 v = (v << 6) | (self.name[i] as u32 & 0x3F);
//                 i += 1;
//                 len -= 1;
//             }
//             // normal hashpjw follows for v
//             h = (h << 4) + v;
//             g = h & 0xf0000000;
//             if g > 0 {
//                 h ^= (g >> 24);
//                 h ^= g;
//             }
//         }
//         state.write_u32(h);
//     }
// }

// static HashValue
// atom_hash(Atom* obj)
// {
//     byte* p = obj->name;
//     int len = obj->len;
//     HashValue h = 0, g;
//     byte v;

//     while(len--) {
//         v = *p++;
//         /* latin1 clutch for r16 */
//         if (len && (v & 0xFE) == 0xC2 && (*p & 0xC0) == 0x80) {
//             v = (v << 6) | (*p & 0x3F);
//             p++; len--;
//         }
//         /* normal hashpjw follows for v */
//         h = (h << 4) + v;
//         if ((g = h & 0xf0000000)) {
//             h ^= (g >> 24);
//             h ^= g;
//         }
//     }
//     return h;
// }

// #[derive(Debug)]
// enum AtomEntryName {
//     Latin1(&'static BStr),
//     Utf8(&'static BStr),
// }

// pub struct Atom<B: 'static + ?Sized, S: 'static + ?Sized> {
//     // slot: IndexSlot,
//     ord0: isize,
//     name: AtomName<B, S>,
// }

// pub enum AtomName<B: 'static + ?Sized, S: 'static + ?Sized> {
//     Latin1(Slice<B>),
//     Utf8(Slice<S>),
// }

struct Interned<T: 'static + ?Sized>(Slice<T>);
impl<T> From<&'static T> for Interned<T>
where
    T: ?Sized,
{
    #[inline]
    fn from(slice: &'static T) -> Self {
        Self(slice.into())
    }
}
impl From<Bytes> for Interned<[u8]> {
    #[inline]
    fn from(owned: Bytes) -> Self {
        Self(owned.into())
    }
}
impl From<BytesMut> for Interned<[u8]> {
    #[inline]
    fn from(owned: BytesMut) -> Self {
        Self(owned.into())
    }
}
impl From<Vec<u8>> for Interned<[u8]> {
    #[inline]
    fn from(owned: Vec<u8>) -> Self {
        Self(owned.into())
    }
}
impl From<Cow<'static, [u8]>> for Interned<[u8]> {
    #[inline]
    fn from(cow: Cow<'static, [u8]>) -> Self {
        Self(cow.into())
    }
}
impl<T> Interned<T>
where
    T: ?Sized,
{
    /// Return a reference to the inner slice.
    #[inline]
    pub const fn as_slice(&self) -> &T {
        self.0.as_slice()
    }

    /// Return a `'static` reference to the inner slice.
    ///
    /// # Safety
    ///
    /// This returns a reference with an unbounded lifetime. It is the caller's
    /// responsibility to make sure it is not used after this `Interned` and its
    /// inner `Slice` is dropped.
    #[inline]
    pub unsafe fn as_static_slice(&self) -> &'static T {
        // Safety:
        //
        // `Interned::as_static_slice`'s caller upheld safety invariants are the
        // same as `Slice::as_static_slice`'s caller upheld safety invariants.
        unsafe { self.0.as_static_slice() }
    }
}
impl<T> Default for Interned<T>
where
    T: ?Sized,
    &'static T: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Slice::default())
    }
}
impl fmt::Debug for Interned<[u8]> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.0)
        } else {
            write!(f, "{:?}", self.0)
        }
    }
}
impl<T> Deref for Interned<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T> PartialEq<Interned<T>> for Interned<T>
where
    T: ?Sized + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T> PartialEq<T> for Interned<T>
where
    T: ?Sized + PartialEq,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_slice() == other
    }
}
impl PartialEq<Vec<u8>> for Interned<[u8]> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl PartialEq<Interned<[u8]>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &Interned<[u8]>) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T> Eq for Interned<T> where T: ?Sized + PartialEq {}
impl<T> Hash for Interned<T>
where
    T: ?Sized + Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}
impl<T> Borrow<T> for Interned<T>
where
    T: ?Sized,
{
    #[inline]
    fn borrow(&self) -> &T {
        self.as_slice()
    }
}
impl<T> Borrow<T> for &Interned<T>
where
    T: ?Sized,
{
    #[inline]
    fn borrow(&self) -> &T {
        self.as_slice()
    }
}
impl<T> AsRef<T> for Interned<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.as_slice()
    }
}

enum Slice<T: 'static + ?Sized> {
    /// True `'static` references.
    Static(&'static T),
    /// Owned `'static` references.
    Owned(Box<T>),
}
impl<T> From<&'static T> for Slice<T>
where
    T: ?Sized,
{
    #[inline]
    fn from(slice: &'static T) -> Self {
        Self::Static(slice)
    }
}
impl From<Bytes> for Slice<[u8]> {
    #[inline]
    fn from(owned: Bytes) -> Self {
        if owned.is_empty() {
            return Self::Static(&b""[..]);
        }
        owned.to_vec().into()
    }
}
impl From<BytesMut> for Slice<[u8]> {
    #[inline]
    fn from(owned: BytesMut) -> Self {
        owned.freeze().into()
    }
}
impl From<Vec<u8>> for Slice<[u8]> {
    #[inline]
    fn from(owned: Vec<u8>) -> Self {
        Self::Owned(owned.into_boxed_slice())
    }
}
impl From<Cow<'static, [u8]>> for Slice<[u8]> {
    #[inline]
    fn from(cow: Cow<'static, [u8]>) -> Self {
        match cow {
            Cow::Borrowed(slice) => slice.into(),
            Cow::Owned(owned) => owned.into(),
        }
    }
}
impl<T> Slice<T>
where
    T: ?Sized,
{
    /// Return a reference to the inner slice.
    #[inline]
    const fn as_slice(&self) -> &T {
        match self {
            Self::Static(slice) => slice,
            Self::Owned(owned) => &**owned,
        }
    }

    /// Return a `'static` reference to the inner slice.
    ///
    /// # Safety
    ///
    /// This returns a reference with an unbounded lifetime. It is the caller's
    /// responsibility to make sure it is not used after this `Slice` is
    /// dropped.
    #[inline]
    unsafe fn as_static_slice(&self) -> &'static T {
        match self {
            Self::Static(slice) => slice,
            Self::Owned(owned) => {
                // Coerce the `Box<T>` to a pointer.
                let ptr: *const T = &**owned;
                // Coerce the pointer to a `&'static T`.
                //
                // Safety:
                //
                // This expression creates a reference with a `'static` lifetime
                // from an owned buffer. This is permissible because:
                //
                // - `Slice` is an internal implementation detail of
                //   `AtomTable` and `bytes::AtomTable`.
                // - `AtomTable` and `bytes::AtomTable` never give out
                //   `'static` references to underlying byte contents.
                // - The `map` field of `AtomTable` and `bytes::AtomTable`,
                //   which contains the `'static` references, is dropped before
                //   the owned buffers stored in this `Slice`.
                unsafe { &*ptr }
            }
        }
    }
}
impl<T> Default for Slice<T>
where
    T: ?Sized,
    &'static T: Default,
{
    #[inline]
    fn default() -> Self {
        Self::Static(<_>::default())
    }
}
impl fmt::Debug for Slice<[u8]> {
    /// Formats the byte slice using the given formatter.
    ///
    /// If alternate format is specified, e.g. `{:#?}`, the slice is assumed to
    /// be conventionally UTF-8 and converted to a [`String`] lossily with
    /// [`String::from_utf8_lossy`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:?}", String::from_utf8_lossy(self.as_slice()))
        } else {
            write!(f, "{:?}", self.as_slice())
        }
    }
}
impl<T> Deref for Slice<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T> PartialEq<Slice<T>> for Slice<T>
where
    T: ?Sized + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T> PartialEq<T> for Slice<T>
where
    T: ?Sized + PartialEq,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_slice() == other
    }
}
impl PartialEq<Vec<u8>> for Slice<[u8]> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl PartialEq<Slice<[u8]>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &Slice<[u8]>) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T> Eq for Slice<T> where T: ?Sized + PartialEq {}
impl<T> Hash for Slice<T>
where
    T: ?Sized + Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}
impl<T> Borrow<T> for Slice<T>
where
    T: ?Sized,
{
    #[inline]
    fn borrow(&self) -> &T {
        self.as_slice()
    }
}
impl<T> Borrow<T> for &Slice<T>
where
    T: ?Sized,
{
    #[inline]
    fn borrow(&self) -> &T {
        self.as_slice()
    }
}
impl<T> AsRef<T> for Slice<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.as_slice()
    }
}
