use async_recursion::async_recursion;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use libflate::zlib;

use std::io::{Read, Write};

use crate::codec::external as ext;
// use crate::dist::{AtomCache, AtomCacheError, AtomRef};
use crate::task::Process;
use crate::term::*;

use super::error::DecodeError;

pub type DecodeResult = Result<Term, DecodeError>;
// pub type DecodeTaskResult = Result<DecodeState<Term>, DecodeError>;

// pub enum DecodeState<T> {
//     Yielded,
//     Complete(T),
// }

// enum DecodeTaskState {
//     ReadExternalTerm(Option<DecodeResult>),
//     ReadCompressedTerm(ReadCompressedTerm),
//     ReadSmallIntegerExt(Option<DecodeResult>),
//     // OuterTerm(&'a Term),
//     // InnerTerm { parent: Option<&'a Term>, child: &'a Term, hint: InnerTermHint, },
//     // InternalFunIterator { term: &'a Term, iter: InternalFunIterator<'a> },
//     // ListIterator { term: &'a Term, iter: ListIterator<'a> },
//     // MapIterator { term: &'a Term, iter: MapIterator<'a> },
//     // TupleIterator { term: &'a Term, iter: TupleIterator<'a> },
// }

// enum ReadCompressedTerm<R> {
//     Enter,
//     Pending { subtask: DecodeTask<R> },
//     Ready(DecodeResult)
// }

// pub struct DecodeTask<R> {
//     task: Task,
//     reader: R,
//     stack: Vec<DecodeTaskState>,
// }
// impl<R: Read> DecodeTask<R> {
//     pub fn new_read_external_term(reader: R, task: Task) -> Self {
//         Self { task, reader, stack: vec![DecodeTaskState::ReadExternalTerm(None)], }
//     }

//     pub fn new_read_internal_term(reader: R, task: Task) -> Self {
//         Self { task, reader, stack: vec![DecodeTaskState::ReadInternalTerm(None)], }
//     }

//     pub fn resume(&mut self) -> DecodeTaskResult {
//         self.task.reset_after_yield();
//         while let Some(state) = self.stack.pop() {
//             match state {
//                 DecodeTaskState::ReadExternalTerm(None) => todo!(),
//                 DecodeTaskState::ReadExternalTerm(Some(Ok(term))) => return Ok(DecodeState::Complete(term)),
//                 DecodeTaskState::ReadExternalTerm(Some(Err(err))) => return Err(err),
//                 DecodeTaskState::ReadCompressedTerm(_) => todo!(),
//                 DecodeTaskState::ReadSmallIntegerExt(_) => todo!(),
//             }
//         }
//         Ok(DecodeState::Yielded)
//     }

//     // fn call(&mut self, func: F) ->  where F: Fn(&mut Self) -> DecodeTaskResult,

//     fn read_external_term(&mut self) -> DecodeTaskResult {
//         let version = self.reader.read_u8()?;
//         if version == ext::VERSION_MAGIC {
//             let tag = self.reader.read_u8()?;
//             match tag {
//                 ext::COMPRESSED => {
//                     self.stack.append(DecodeTaskState::ReadCompressedTerm(ReadCompressedTerm::Enter));
//                     let _uncompressed_size = self.reader.read_u32::<BigEndian>()? as usize;
//                     let zlib_decoder = zlib::Decoder::new(&mut self.reader)?;
//                     let decoder_task = DecoderTask::new_read_internal_term(Task::spawn(self.task), zlib_decoder);
//                     self.stack.append(DecodeTaskState::ReadCompressedTerm()))
//                     Ok(DecodeState::Yielded)
//                 }
//             }
//         } else {
//             Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid version magic"))
//         }
//     }
// }

// #[derive(Clone, Debug)]
// struct AtomCacheRef<'a> {
//     internal_index: usize,
//     atom: &'a Atom,
// }

pub struct ReadContext<'a> {
    pub process: &'a Process,
    pub atom_cache: &'a AtomCache,
    atom_cache_refs: Vec<AtomRef>,
}
impl<'a> ReadContext<'a> {
    pub fn new(process: &'a Process, atom_cache: &'a AtomCache) -> Self {
        Self { process, atom_cache, atom_cache_refs: vec![], }
    }

    pub async fn bump_all_reds(&self) {
        self.process.bump_all_reds().await;
    }

    pub async fn bump_reds(&self, gc: isize) {
        self.process.bump_reds(gc).await;
    }
}

pub struct YieldableDecoder<R> {
    // process: Process,
    reader: R,
    // atom_cache_refs: Vec<AtomCacheRef<'a>>,
    // atom_cache_refs: Vec<AtomRef>,
    // atom_cache: Option<AtomCache>,
    buf: Vec<u8>,
}
impl<R: Read> YieldableDecoder<R> {
    // pub fn new(process: Process, reader: R, atom_cache: Option<AtomCache>) -> Self {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: vec![],
        }
        //     Self {
        //     process,
        //     reader,
        //     atom_cache_refs: Vec::new(),
        //     atom_cache,
        //     buf: vec![],
        // }
    }

    pub async fn read_external_term(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let version = self.reader.read_u8()?;
        if version == ext::VERSION_MAGIC {
            let tag = self.reader.read_u8()?;
            match tag {
                ext::COMPRESSED => self.read_compressed_term(ctx).await,
                _ => self.read_internal_term_with_tag(ctx, tag).await,
            }
        } else {
            Err(DecodeError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid version magic",
            )))
        }
    }

    async fn read_compressed_term(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let _uncompressed_size = self.reader.read_u32::<BigEndian>()? as usize;
        let zlib_decoder = zlib::Decoder::new(&mut self.reader)?;
        // FIXME: support compressed atom cache decoding
        let mut decoder = YieldableDecoder::new(zlib_decoder);
        decoder.read_internal_term(ctx).await
    }

    pub async fn read_internal_term(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let tag = self.reader.read_u8()?;
        self.read_internal_term_with_tag(tag).await
    }

    async fn read_internal_term_with_tag(&mut self, ctx: &ReadContext<'_>, tag: u8) -> DecodeResult {
        match tag {
            ext::SMALL_INTEGER_EXT => self.read_small_integer_ext(ctx),
            ext::INTEGER_EXT => self.read_integer_ext(ctx),
            ext::FLOAT_EXT => self.read_float_ext(ctx),
            ext::ATOM_EXT => self.read_atom_ext(ctx),
            ext::SMALL_ATOM_EXT => self.read_small_atom_ext(ctx),
            ext::REFERENCE_EXT => self.read_reference_ext(ctx).await,
            ext::NEW_REFERENCE_EXT => self.read_new_reference_ext(ctx).await,
            ext::NEWER_REFERENCE_EXT => self.read_newer_reference_ext(ctx).await,
            ext::PORT_EXT => self.read_port_ext(ctx).await,
            ext::NEW_PORT_EXT => self.read_new_port_ext(ctx).await,
            ext::NEW_FLOAT_EXT => self.read_new_float_ext(ctx),
            ext::PID_EXT => self.read_pid_ext(ctx).await,
            ext::NEW_PID_EXT => self.read_new_pid_ext(ctx).await,
            ext::SMALL_TUPLE_EXT => self.read_small_tuple_ext(ctx).await,
            ext::LARGE_TUPLE_EXT => self.read_large_tuple_ext(ctx).await,
            ext::NIL_EXT => self.read_nil_ext(ctx),
            ext::STRING_EXT => self.read_string_ext(ctx).await,
            ext::LIST_EXT => self.read_list_ext(ctx).await,
            ext::BINARY_EXT => self.read_binary_ext(ctx).await,
            ext::BIT_BINARY_EXT => self.read_bit_binary_ext(ctx).await,
            ext::SMALL_BIG_EXT => self.read_small_big_ext(ctx).await,
            ext::LARGE_BIG_EXT => self.read_large_big_ext(ctx).await,
            ext::NEW_FUN_EXT => self.read_new_fun_ext(ctx).await,
            ext::EXPORT_EXT => self.read_export_ext(ctx).await,
            ext::MAP_EXT => self.read_map_ext(ctx).await,
            ext::FUN_EXT => self.read_fun_ext(ctx).await,
            ext::ATOM_UTF8_EXT => self.read_atom_utf8_ext(ctx),
            ext::SMALL_ATOM_UTF8_EXT => self.read_small_atom_utf8_ext(ctx),
            ext::V4_PORT_EXT => self.read_v4_port_ext(ctx).await,

            ext::DIST_HEADER => self.read_dist_header(ctx).await,
            // ext::DIST_FRAG_HEADER => self.read_dist_frag_header().await,
            // ext::DIST_FRAG_CONT => self.read_dist_frag_cont().await,
            ext::ATOM_CACHE_REF => self.read_atom_cache_ref(ctx),

            _ => {
                println!("tag was {:?}", tag);
                Err(DecodeError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "expected internal term",
                )))
            }
        }
    }

    async fn read_internal_atom(&mut self, ctx: &ReadContext<'_>) -> Result<Atom, DecodeError> {
        ctx.bump_reds(1).await;
        let tag = self.reader.read_u8()?;
        let term = match tag {
            ext::ATOM_EXT => self.read_atom_ext(ctx),
            ext::SMALL_ATOM_EXT => self.read_small_atom_ext(ctx),
            ext::ATOM_UTF8_EXT => self.read_atom_utf8_ext(ctx),
            ext::SMALL_ATOM_UTF8_EXT => self.read_small_atom_utf8_ext(ctx),
            ext::ATOM_CACHE_REF => self.read_atom_cache_ref(ctx),
            _ => Err(DecodeError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected internal atom",
            ))),
        }?;
        aux::term_into_atom(term)
    }

    async fn read_internal_i32(&mut self, ctx: &ReadContext<'_>) -> Result<i32, DecodeError> {
        ctx.bump_reds(1).await;
        let tag = self.reader.read_u8()?;
        let term = match tag {
            ext::SMALL_INTEGER_EXT => self.read_small_integer_ext(ctx),
            ext::INTEGER_EXT => self.read_integer_ext(ctx),
            _ => Err(DecodeError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected internal integer",
            ))),
        }?;
        aux::term_into_i32(term)
    }

    async fn read_internal_pid(&mut self, ctx: &ReadContext<'_>) -> Result<Pid, DecodeError> {
        ctx.bump_reds(1).await;
        let tag = self.reader.read_u8()?;
        let term = match tag {
            ext::PID_EXT => self.read_pid_ext(ctx).await,
            ext::NEW_PID_EXT => self.read_new_pid_ext(ctx).await,
            _ => Err(DecodeError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected internal pid",
            ))),
        }?;
        aux::term_into_pid(term)
    }

    async fn read_internal_u8(&mut self, ctx: &ReadContext<'_>) -> Result<u8, DecodeError> {
        ctx.bump_reds(1).await;
        let tag = self.reader.read_u8()?;
        let term = match tag {
            ext::SMALL_INTEGER_EXT => self.read_small_integer_ext(ctx),
            _ => Err(DecodeError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected internal small integer",
            ))),
        }?;
        aux::term_into_i32(term).and_then(|x| Ok(x as u8))
    }

    fn read_small_integer_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        let value = self.reader.read_u8()?;
        Ok(Term::from(Number::from(FixInteger::from(value))))
    }

    fn read_integer_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        let value = self.reader.read_i32::<BigEndian>()?;
        Ok(Term::from(Number::from(FixInteger::from(value))))
    }

    fn read_float_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        let mut buf = [0; 31];
        self.reader.read_exact(&mut buf)?;
        let float_str = std::str::from_utf8(&buf)
            .or_else(|e| aux::invalid_data_error(e.to_string()))?
            .trim_end_matches(0 as char);
        let value = float_str
            .parse::<f32>()
            .or_else(|e| aux::invalid_data_error(e.to_string()))?;
        Ok(Term::from(Number::from(Float::try_from(value)?)))
    }

    fn read_atom_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        let len = self.reader.read_u16::<BigEndian>()?;
        self.buf.resize(len as usize, 0);
        self.reader.read_exact(&mut self.buf)?;
        let name = aux::latin1_bytes_to_string(&self.buf)?;
        Ok(Term::from(Atom::from(name)))
    }

    fn read_small_atom_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        let len = self.reader.read_u8()?;
        self.buf.resize(len as usize, 0);
        self.reader.read_exact(&mut self.buf)?;
        let name = aux::latin1_bytes_to_string(&self.buf)?;
        Ok(Term::from(Atom::from(name)))
    }

    async fn read_reference_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let node = self.read_internal_atom(ctx).await?;
        let id = vec![self.reader.read_u32::<BigEndian>()?];
        let creation = u32::from(self.reader.read_u8()?);
        Ok(Term::from(Reference::new(node, id, creation)))
    }

    async fn read_new_reference_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let id_count = self.reader.read_u16::<BigEndian>()? as usize;
        let node = self.read_internal_atom(ctx).await?;
        let creation = u32::from(self.reader.read_u8()?);
        let mut id = Vec::with_capacity(id_count);
        for _ in 0..id_count {
            id.push(self.reader.read_u32::<BigEndian>()?);
        }
        Ok(Term::from(Reference::new(node, id, creation)))
    }

    async fn read_newer_reference_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let id_count = self.reader.read_u16::<BigEndian>()? as usize;
        let node = self.read_internal_atom(ctx).await?;
        let creation = self.reader.read_u32::<BigEndian>()?;
        let mut id = Vec::with_capacity(id_count);
        for _ in 0..id_count {
            id.push(self.reader.read_u32::<BigEndian>()?);
        }
        Ok(Term::from(Reference::new(node, id, creation)))
    }

    async fn read_port_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let node = self.read_internal_atom(ctx).await?;
        let id = u64::from(self.reader.read_u32::<BigEndian>()?);
        let creation = u32::from(self.reader.read_u8()?);
        Ok(Term::from(Port::new(node, id, creation)))
    }

    async fn read_new_port_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let node = self.read_internal_atom(ctx).await?;
        let id = u64::from(self.reader.read_u32::<BigEndian>()?);
        let creation = self.reader.read_u32::<BigEndian>()?;
        Ok(Term::from(Port::new(node, id, creation)))
    }

    fn read_new_float_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        let value = self.reader.read_f64::<BigEndian>()?;
        Ok(Term::from(Number::from(Float::try_from(value)?)))
    }

    async fn read_pid_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let node = self.read_internal_atom(ctx).await?;
        let id = self.reader.read_u32::<BigEndian>()?;
        let serial = self.reader.read_u32::<BigEndian>()?;
        let creation = u32::from(self.reader.read_u8()?);
        Ok(Term::from(Pid::new(node, id, serial, creation)))
    }

    async fn read_new_pid_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let node = self.read_internal_atom(ctx).await?;
        let id = self.reader.read_u32::<BigEndian>()?;
        let serial = self.reader.read_u32::<BigEndian>()?;
        let creation = self.reader.read_u32::<BigEndian>()?;
        Ok(Term::from(Pid::new(node, id, serial, creation)))
    }

    #[async_recursion(?Send)]
    async fn read_small_tuple_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let arity = self.reader.read_u8()? as usize;
        let mut elements = Vec::with_capacity(arity);
        for _ in 0..arity {
            elements.push(self.read_internal_term(ctx).await?);
        }
        Ok(Term::from(Tuple::from(elements)))
    }

    #[async_recursion(?Send)]
    async fn read_large_tuple_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let arity = self.reader.read_u32::<BigEndian>()? as usize;
        let mut elements = Vec::with_capacity(arity);
        for _ in 0..arity {
            elements.push(self.read_internal_term(ctx).await?);
        }
        Ok(Term::from(Tuple::from(elements)))
    }

    fn read_nil_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        Ok(Term::from(Nil))
    }

    async fn read_string_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let len = self.reader.read_u16::<BigEndian>()? as usize;
        let mut elements = Vec::with_capacity(len);
        for i in 0..len {
            elements.push(Term::from(Number::from(FixInteger::from(i32::from(
                self.reader.read_u8()?,
            )))));
            if i > 0 && i % 4096 == 0 {
                ctx.bump_reds(5).await;
            }
        }
        Ok(Term::from(List::from(elements)))
    }

    #[async_recursion(?Send)]
    async fn read_list_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let len = self.reader.read_u32::<BigEndian>()? as usize;
        let mut elements = Vec::with_capacity(len);
        for _ in 0..len {
            elements.push(self.read_internal_term(ctx).await?);
        }
        let tail = self.read_internal_term(ctx).await?;
        Ok(Term::from(List::from((elements, tail))))
    }

    async fn read_binary_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let len = self.reader.read_u32::<BigEndian>()? as usize;
        let mut buf = vec![0; len];
        self.reader.read_exact(&mut buf)?;
        Ok(Term::from(Bitstring::from(buf)))
    }

    async fn read_bit_binary_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let len = self.reader.read_u32::<BigEndian>()? as usize;
        let tail_bits = self.reader.read_u8()?;
        let mut buf = vec![0; len];
        self.reader.read_exact(&mut buf)?;
        if !buf.is_empty() {
            let tail = buf[len - 1] >> (8 - tail_bits);
            buf[len - 1] = tail;
        }
        Ok(Term::from(Bitstring::from((buf, tail_bits))))
    }

    async fn read_small_big_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let n = self.reader.read_u8()? as usize;
        let sign = self.reader.read_u8()?;
        self.buf.resize(n, 0);
        self.reader.read_exact(&mut self.buf)?;
        let value = BigInt::from_bytes_le(aux::byte_to_sign(sign)?, &self.buf);
        Ok(Term::from(Number::from(Bignum { value })))
    }

    async fn read_large_big_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let n = self.reader.read_u32::<BigEndian>()? as usize;
        let sign = self.reader.read_u8()?;
        self.buf.resize(n, 0);
        self.reader.read_exact(&mut self.buf)?;
        let value = BigInt::from_bytes_le(aux::byte_to_sign(sign)?, &self.buf);
        Ok(Term::from(Number::from(Bignum { value })))
    }

    #[async_recursion(?Send)]
    async fn read_new_fun_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let _size = self.reader.read_u32::<BigEndian>()?;
        let arity = self.reader.read_u8()?;
        let mut uniq = [0; 16];
        self.reader.read_exact(&mut uniq)?;
        let index = self.reader.read_u32::<BigEndian>()?;
        let num_free = self.reader.read_u32::<BigEndian>()?;
        let module = self.read_internal_atom(ctx).await?;
        let old_index = self.read_internal_i32(ctx).await?;
        let old_uniq = self.read_internal_i32(ctx).await?;
        let pid = self.read_internal_pid(ctx).await?;
        let mut free_vars = Vec::with_capacity(num_free as usize);
        for _ in 0..num_free {
            free_vars.push(self.read_internal_term(ctx).await?);
        }
        Ok(Term::from(Fun::from(InternalFun::New {
            module,
            arity,
            pid,
            free_vars,
            index,
            uniq,
            old_index,
            old_uniq,
        })))
    }

    async fn read_export_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let module = self.read_internal_atom(ctx).await?;
        let function = self.read_internal_atom(ctx).await?;
        let arity = self.read_internal_u8(ctx).await?;
        Ok(Term::from(Fun::from(ExternalFun {
            module,
            function,
            arity,
        })))
    }

    #[async_recursion(?Send)]
    async fn read_map_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let arity = self.reader.read_u32::<BigEndian>()? as usize;
        let mut pairs = Vec::with_capacity(arity);
        for _ in 0..arity {
            let key = self.read_internal_term(ctx).await?;
            let val = self.read_internal_term(ctx).await?;
            pairs.push((key, val));
        }
        Ok(Term::from(Map::from(pairs)))
    }

    #[async_recursion(?Send)]
    async fn read_fun_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let num_free = self.reader.read_u32::<BigEndian>()?;
        let pid = self.read_internal_pid(ctx).await?;
        let module = self.read_internal_atom(ctx).await?;
        let index = self.read_internal_i32(ctx).await?;
        let uniq = self.read_internal_i32(ctx).await?;
        let mut free_vars = Vec::with_capacity(num_free as usize);
        for _ in 0..num_free {
            free_vars.push(self.read_internal_term(ctx).await?);
        }
        Ok(Term::from(Fun::from(InternalFun::Old {
            module,
            pid,
            free_vars,
            index,
            uniq,
        })))
    }

    fn read_atom_utf8_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        let len = self.reader.read_u16::<BigEndian>()?;
        self.buf.resize(len as usize, 0);
        self.reader.read_exact(&mut self.buf)?;
        let name =
            std::str::from_utf8(&self.buf).or_else(|e| aux::invalid_data_error(e.to_string()))?;
        Ok(Term::from(Atom::from(name)))
    }

    fn read_small_atom_utf8_ext(&mut self, _ctx: &ReadContext<'_>) -> DecodeResult {
        let len = self.reader.read_u8()?;
        self.buf.resize(len as usize, 0);
        self.reader.read_exact(&mut self.buf)?;
        let name =
            std::str::from_utf8(&self.buf).or_else(|e| aux::invalid_data_error(e.to_string()))?;
        Ok(Term::from(Atom::from(name)))
    }

    async fn read_v4_port_ext(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        ctx.bump_reds(1).await;
        let node = self.read_internal_atom(ctx).await?;
        let id = self.reader.read_u64::<BigEndian>()?;
        let creation = self.reader.read_u32::<BigEndian>()?;
        Ok(Term::from(Port::new(node, id, creation)))
    }

    async fn read_dist_header(&mut self, ctx: &ReadContext<'_>) -> DecodeResult {
        let number_of_atom_cache_refs = self.reader.read_u8()?;
        // self.reader.read_
        if number_of_atom_cache_refs > 0 {
            let flags_len = (number_of_atom_cache_refs as usize / 2) + 1;
            let mut flags_buf = vec![0; flags_len];
            self.reader.read_exact(&mut flags_buf)?;
            use bitter::{BigEndianReader, BitReader};
            let mut flags_reader = BigEndianReader::new(&flags_buf[..]);
            // #[derive(Debug)]
            // struct AtomCacheEntry {
            //     atom_cache_reference_index: usize,
            //     new_cache_entry_flag: bool,
            //     segment_index: u64,
            //     update: AtomCacheRefUpdate,
            // }
            // #[derive(Debug)]
            // enum AtomCacheRefUpdate {
            //     Old {
            //         internal_segment_index: u64,
            //     },
            //     New {
            //         internal_segment_index: u64,
            //         atom_text: Vec<u8>,
            //     }
            // }
            // let mut entries = Vec::with_capacity(number_of_atom_cache_refs as usize);
            let flags_buf_last_byte = flags_buf.last().unwrap();
            let long_atoms = if number_of_atom_cache_refs % 2 == 0 {
                (flags_buf_last_byte & 1) == 1
            } else {
                ((flags_buf_last_byte >> 4) & 1) == 1
            };
            let atom_cache = self.atom_cache.clone().unwrap();
            // println!("bits remaining: {:?}", flags_reader.bits_remaining());
            for atom_cache_reference_index in 0..(number_of_atom_cache_refs as usize) {
                let new_cache_entry_flag = flags_reader.read_bit_unchecked();
                let segment_index = flags_reader.read_bits_unchecked(3);
                // let _currently_unused = flags_reader.read_bits_max_unchecked(3);
                // let long_atoms = flags_reader.read_bit_unchecked();
                let internal_segment_index = self.reader.read_u8()? as u64;
                if new_cache_entry_flag {
                    let atom_len = if long_atoms { self.reader.read_u16::<BigEndian>()? as usize } else { self.reader.read_u8()? as usize };
                    let mut atom_text = vec![0; atom_len];
                    self.reader.read_exact(&mut atom_text)?;
                    // FIXME: support UTF-8 and Latin1 based on DFLAG_UTF8_ATOMS
                    let atom_ref = AtomRef::new(Atom::from(String::from_utf8(atom_text).or_else(|e| aux::invalid_data_error(e.to_string()))?));
                    // aux::latin1_bytes_to_string(&self.buf)?;
                    let internal_index = ((segment_index << 8) | internal_segment_index) as usize;
                    atom_cache.insert(internal_index, atom_ref.clone())?;
                    self.atom_cache_refs.push(atom_ref);
                    // atom_cache.get(internal_index).unwrap().cloned();
                    // self.atom_cache_refs.push(&atom_cache.get(internal_index).unwrap().as_ref().unwrap());
                // let internal_segment_index = match {
                //     AtomCacheRefUpdate::New {}
                // };
                // let internal_index = entry.segment_index
                // self.atom_cache_refs.push(AtomCacheRef {})
                //     entries.push(AtomCacheEntry {
                //         atom_cache_reference_index,
                //         new_cache_entry_flag,
                //         segment_index,
                //         update: AtomCacheRefUpdate::New { internal_segment_index, atom_text, },
                //     });
                } else {
                    let internal_index = ((segment_index << 8) | internal_segment_index) as usize;
                    let atom_ref = atom_cache.get(internal_index)?.ok_or_else(|| aux::invalid_data_error::<DecodeResult>(format!("atom cache not found: {:?}", internal_index)).unwrap_err())?;
                    self.atom_cache_refs.push(atom_ref);
                    // self.atom_cache_refs.push(&atom_cache.get(internal_index).unwrap().as_ref().unwrap());
                    // entries.push(AtomCacheEntry {
                    //     atom_cache_reference_index,
                    //     new_cache_entry_flag,
                    //     segment_index,
                    //     update: AtomCacheRefUpdate::Old { internal_segment_index, },
                    // });
                }
                // flags.push(Flag {
                //     atom_cache_reference_index,
                //     new_cache_entry_flag,
                //     segment_index,
                //     long_atoms,
                // });
                // println!("[flags:{:?}] {:?} {:?} {:?}", i, new_cache_entry_flag, segment_index, long_atoms);
            }
            println!("atom_cache_refs = {:?}", self.atom_cache_refs);
            println!("atom_cache = {:?}", self.atom_cache);
            // println!("bits remaining: {:?}", flags_reader.approx_bytes_remaining());
            // println!("flags = {:?}", entries);
            // for entry in entries {
            //     println!("{:?}", entry);
            // }
            // for entry in entries {
            //     let mut atom_cache = self.atom_cache.unwrap();
            //     let internal_segment_index = match {
            //         AtomCacheRefUpdate::New {}
            //     };
            //     let internal_index = entry.segment_index
            //     self.atom_cache_refs.push(AtomCacheRef {})
            // }
        }
        Ok(Term::Nil(Nil))
    }

    // async fn read_dist_frag_header(&mut self) -> DecodeResult {
    //     let sequence_id = self.reader.read_u64::<BigEndian>()?;
    //     let fragment_id = self.reader.read_u64::<BigEndian>()?;
    //     let number_of_atom_cache_refs = self.reader.read_u8()?;
    //     let flags_len = if number_of_atom_cache_refs == 0 {
    //         0
    //     } else {
    //         (number_of_atom_cache_refs as usize / 2) + 1
    //     };
    //     let mut flags_buf = vec![0; flags_len];
    //     self.reader.read_exact(&mut flags_buf)?;
    //     Ok(Term::Nil(Nil))
    // }

    fn read_atom_cache_ref(&mut self) -> DecodeResult {
        let atom_cache_reference_index = self.reader.read_u8()? as usize;
        if let Some(atom_ref) = self.atom_cache_refs.get(atom_cache_reference_index) {
            let atom = atom_ref.to_owned_atom();
            Ok(Term::from(atom))
        } else {
            aux::invalid_data_error(format!("no atom cache ref index found {:?}", atom_cache_reference_index)).map_err(|err| err.into())
        }
    }
}

mod aux {
    use crate::term::Sign;

    pub fn byte_to_sign(b: u8) -> std::io::Result<Sign> {
        match b {
            0 => Ok(Sign::Plus),
            1 => Ok(Sign::Minus),
            _ => invalid_data_error(format!("A sign value must be 0 or 1: value={}", b)),
        }
    }

    pub fn invalid_data_error<T>(message: String) -> std::io::Result<T> {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        ))
    }

    pub fn latin1_bytes_to_string(buf: &[u8]) -> std::io::Result<String> {
        // FIXME: Supports Latin1 characters
        std::str::from_utf8(buf)
            .or_else(|e| other_error(e.to_string()))
            .map(ToString::to_string)
    }

    pub fn other_error<T>(message: String) -> std::io::Result<T> {
        Err(std::io::Error::new(std::io::ErrorKind::Other, message))
    }

    pub fn term_into_atom(t: crate::Term) -> Result<crate::term::Atom, super::DecodeError> {
        t.try_into()
            .map_err(|t| super::DecodeError::UnexpectedType {
                value: t,
                expected: "Atom".to_string(),
            })
    }

    pub fn term_into_i32(t: crate::Term) -> Result<i32, super::DecodeError> {
        t.try_into()
            .map_err(|t| super::DecodeError::UnexpectedType {
                value: t,
                expected: "FixInteger".to_string(),
            })
    }

    pub fn term_into_pid(t: crate::Term) -> Result<crate::term::Pid, super::DecodeError> {
        t.try_into()
            .map_err(|t| super::DecodeError::UnexpectedType {
                value: t,
                expected: "Pid".to_string(),
            })
    }
}
