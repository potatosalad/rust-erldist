/// See [erts/emulator/beam/external.h](https://github.com/erlang/otp/blob/OTP-25.0.3/erts/emulator/beam/external.h#L31-L82) in the Erlang/OTP source code.

pub(crate) const SMALL_INTEGER_EXT: u8 = 'a' as u8;
pub(crate) const INTEGER_EXT: u8 = 'b' as u8;
pub(crate) const FLOAT_EXT: u8 = 'c' as u8;
pub(crate) const ATOM_EXT: u8 = 'd' as u8;
pub(crate) const SMALL_ATOM_EXT: u8 = 's' as u8;
pub(crate) const REFERENCE_EXT: u8 = 'e' as u8;
pub(crate) const NEW_REFERENCE_EXT: u8 = 'r' as u8;
pub(crate) const NEWER_REFERENCE_EXT: u8 = 'Z' as u8;
pub(crate) const PORT_EXT: u8 = 'f' as u8;
pub(crate) const NEW_PORT_EXT: u8 = 'Y' as u8;
pub(crate) const NEW_FLOAT_EXT: u8 = 'F' as u8;
pub(crate) const PID_EXT: u8 = 'g' as u8;
pub(crate) const NEW_PID_EXT: u8 = 'X' as u8;
pub(crate) const SMALL_TUPLE_EXT: u8 = 'h' as u8;
pub(crate) const LARGE_TUPLE_EXT: u8 = 'i' as u8;
pub(crate) const NIL_EXT: u8 = 'j' as u8;
pub(crate) const STRING_EXT: u8 = 'k' as u8;
pub(crate) const LIST_EXT: u8 = 'l' as u8;
pub(crate) const BINARY_EXT: u8 = 'm' as u8;
pub(crate) const BIT_BINARY_EXT: u8 = 'M' as u8;
pub(crate) const SMALL_BIG_EXT: u8 = 'n' as u8;
pub(crate) const LARGE_BIG_EXT: u8 = 'o' as u8;
pub(crate) const NEW_FUN_EXT: u8 = 'p' as u8;
pub(crate) const EXPORT_EXT: u8 = 'q' as u8;
pub(crate) const MAP_EXT: u8 = 't' as u8;
pub(crate) const FUN_EXT: u8 = 'u' as u8;
pub(crate) const ATOM_UTF8_EXT: u8 = 'v' as u8;
pub(crate) const SMALL_ATOM_UTF8_EXT: u8 = 'w' as u8;
pub(crate) const V4_PORT_EXT: u8 = 'x' as u8;

pub(crate) const DIST_HEADER: u8 = 'D' as u8;
pub(crate) const DIST_FRAG_HEADER: u8 = 'E' as u8;
pub(crate) const DIST_FRAG_CONT: u8 = 'F' as u8;
// pub(crate) const HOPEFUL_DATA: u8 = 'H' as u8;
pub(crate) const ATOM_CACHE_REF: u8 = 'R' as u8;
// pub(crate) const ATOM_INTERNAL_REF2: u8 = 'I' as u8;
// pub(crate) const ATOM_INTERNAL_REF3: u8 = 'K' as u8;
// pub(crate) const BINARY_INTERNAL_REF: u8 = 'J' as u8;
// pub(crate) const BIT_BINARY_INTERNAL_REF: u8 = 'L' as u8;
pub(crate) const COMPRESSED: u8 = 'P' as u8;

pub(crate) const VERSION_MAGIC: u8 = 131 as u8;

pub(crate) const ERTS_ATOM_CACHE_SIZE: usize = 2048;
pub(crate) const ERTS_USE_ATOM_CACHE_SIZE: usize = 2039;
pub(crate) const ERTS_MAX_INTERNAL_ATOM_CACHE_ENTRIES: usize = 255;
