pub use num_bigint;

pub mod codec;
pub mod dist;
pub mod env;
pub mod task;
pub mod term;

// pub use codec::*;
pub use task::*;
pub use term::Term;

pub use bytes;
pub use cassette::{pin_mut, Cassette};

// pub struct Env(EnvInner);

// enum EnvInner {
//     Blocking,
//     ReductionCounting(Cell<isize>),
// }

pub async fn foo_1(p: &Process, arg: u32) -> Result<u32, std::io::Error> {
    let x = foo_2(p, arg).await?;
    p.bump_reds(1).await;
    Ok(x)
}

pub async fn foo_2(p: &Process, arg: u32) -> Result<u32, std::io::Error> {
    let mut reds = 0;
    for x in 0..arg {
        let part_reds = foo_3(p, x).await?;
        p.bump_reds(part_reds as isize).await;
        reds += part_reds;
    }
    Ok(reds)
}

pub async fn foo_3(p: &Process, arg: u32) -> Result<u32, std::io::Error> {
    p.bump_reds(arg as isize * 10).await;
    if arg % 7 == 0 {
        Ok(p.get_remaining_reds() as u32)
    } else {
        Ok(arg * 13)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::*;
        // let process = std::sync::Arc::new(std::cell::RefCell::new(Process::new()));
        let process = Process::yielding();
        // let future = foo_1(&process, 18);

        // float
        // let buf: Vec<u8> = vec![131,70,63,243,51,51,51,51,51,51];

        // fun
        // let buf: Vec<u8> = vec![131,112,0,0,2,247,0,125,102,234,72,192,153,55,197,5,22,1,37,249,98,108,111,0,0,0,45,0,0,0,1,100,0,8,101,114,108,95,101,118,97,108,97,45,98,3,235,55,82,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,79,0,0,0,0,0,0,0,0,104,4,106,104,2,100,0,4,101,118,97,108,112,0,0,1,245,3,0,177,9,183,184,199,48,15,82,217,180,63,32,65,218,189,0,0,0,21,0,0,0,4,100,0,5,115,104,101,108,108,97,21,98,0,5,136,77,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,79,0,0,0,0,0,0,0,0,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,73,0,0,0,0,0,0,0,0,90,0,3,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,0,0,3,47,60,6,214,0,3,122,178,8,15,104,2,100,0,5,118,97,108,117,101,112,0,0,0,102,2,0,177,9,183,184,199,48,15,82,217,180,63,32,65,218,189,0,0,0,5,0,0,0,1,100,0,5,115,104,101,108,108,97,5,98,0,5,136,77,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,79,0,0,0,0,0,0,0,0,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,73,0,0,0,0,0,0,0,0,112,0,0,0,250,1,0,177,9,183,184,199,48,15,82,217,180,63,32,65,218,189,0,0,0,12,0,0,0,3,100,0,5,115,104,101,108,108,97,12,98,0,5,136,77,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,79,0,0,0,0,0,0,0,0,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,73,0,0,0,0,0,0,0,0,90,0,3,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,0,0,3,47,60,6,214,0,3,122,178,8,15,104,2,100,0,5,118,97,108,117,101,112,0,0,0,102,2,0,177,9,183,184,199,48,15,82,217,180,63,32,65,218,189,0,0,0,5,0,0,0,1,100,0,5,115,104,101,108,108,97,5,98,0,5,136,77,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,79,0,0,0,0,0,0,0,0,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,73,0,0,0,0,0,0,0,0,104,2,100,0,5,118,97,108,117,101,112,0,0,0,102,2,0,177,9,183,184,199,48,15,82,217,180,63,32,65,218,189,0,0,0,5,0,0,0,1,100,0,5,115,104,101,108,108,97,5,98,0,5,136,77,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,79,0,0,0,0,0,0,0,0,88,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,73,0,0,0,0,0,0,0,0,108,0,0,0,1,104,5,100,0,6,99,108,97,117,115,101,104,2,97,1,97,46,106,106,108,0,0,0,1,104,3,100,0,5,102,108,111,97,116,104,2,97,1,97,52,70,63,243,51,51,51,51,51,51,106,106];

        // reference
        // let buf: Vec<u8> = vec![131,90,0,3,100,0,13,110,111,110,111,100,101,64,110,111,104,111,115,116,0,0,0,0,0,3,47,106,6,212,0,3,122,178,8,15];

        // small list (11)
        // let buf: Vec<u8> = vec![131,108,0,0,0,11,98,0,0,1,44,98,0,0,1,45,98,0,0,1,46,98,0,0,1,47,98,0,0,1,48,98,0,0,1,49,98,0,0,1,50,98,0,0,1,51,98,0,0,1,52,98,0,0,1,53,98,0,0,1,54,106];

        // string
        // let buf: Vec<u8> = vec![131,107,0,10,1,2,3,4,5,6,7,8,9,10];

        // nil
        // let buf: Vec<u8> = vec![131,106];

        // compressed binary
        // let buf = vec![131,80,0,0,4,5,120,156,203,101,96,96,97,24,5,163,96,20,140,92,0,0,202,61,0,114];

        // compressed bit binary
        // let buf = vec![131,80,0,0,4,7,120,156,243,101,96,96,97,100,100,24,5,163,96,20,140,88,208,0,0,82,198,0,212];

        // dist header
        // let buf = vec![131,68,5,4,137,9,10,5,236,3,114,101,103,9,4,99,97,108,108,238,13,115,101,116,95,103,101,116,95,115,116,97,116,101];

        // let mut atom_cache = crate::dist::AtomCache::new();
        // atom_cache.insert(10, crate::dist::AtomRef::new(crate::term::Atom::from("atom10"))).unwrap();
        // atom_cache.insert(238, crate::dist::AtomRef::new(crate::term::Atom::from("atom238"))).unwrap();
        // atom_cache.insert(1029, crate::dist::AtomRef::new(crate::term::Atom::from("atom1029"))).unwrap();
        // let reader = std::io::Cursor::new(buf);
        // let mut decoder = crate::codec::decoder::YieldableDecoder::new(process.clone(), reader, Some(atom_cache.clone()));
        // let future = decoder.read_external_term();
        // pin_mut!(future);
        // let mut cm = Cassette::new(future);
        // loop {
        //     match cm.poll_on() {
        //         Some(result) => {
        //             // match result {
        //             //     Ok(t) => {
        //             //         println!("cassette ok: {}", t);
        //             //     }
        //             //     Err(e) => {
        //             //         println!("cassette err: {:?}", e);
        //             //     }
        //             // };
        //             println!("cassette returned {:?}", result);
        //             println!(
        //                 "\t[reds] con = {:?}, rem = {:?}, pct = {:?}",
        //                 process.get_consumed_reds(),
        //                 process.get_remaining_reds(),
        //                 process.get_timeslice_pct()
        //             );
        //             break;
        //         }
        //         None => {
        //             println!("yielded");
        //             println!(
        //                 "\t[reds] con = {:?}, rem = {:?}, pct = {:?}",
        //                 process.get_consumed_reds(),
        //                 process.get_remaining_reds(),
        //                 process.get_timeslice_pct()
        //             );
        //         }
        //     }
        // }

        // println!("\n\n{:?}", atom_cache);

        // use crate::term::Atom;
        // let mut cm = crate::term::AtomCacheIndexer::new();
        // let atoms = vec![
        //     Atom::from(""),
        //     Atom::from("foo"),
        //     Atom::from("bar"),
        //     Atom::from("baz"),
        // ];
        // for atom in atoms.iter() {
        //     cm.insert(atom);
        // }
        // for atom in atoms.iter() {
        //     println!("{:?} = {:?}", atom, cm.get(atom));
        // }
        // println!("{:?}", cm);

        // use crate::term::table::{Atom, AtomTable};
        // let mut at = AtomTable::new();
        // let atoms = vec![
        //     Atom::from(""),
        //     Atom::from("foo"),
        //     Atom::from("bar"),
        //     Atom::from("baz"),
        // ];
        // for atom in atoms.iter() {
        //     at.get_or_insert(atom);
        // }
        // println!("{:?}", at);

        use crate::env::{Env};
        use crate::bytes::Bytes;
        // use std::mem::ManuallyDrop;

        // struct MyString(ManuallyDrop<String>);
        // impl MyString {
        //     fn new(x: String) -> Self {
        //         Self(ManuallyDrop::new(x))
        //     }

        //     fn as_bytes(&self) -> &[u8] {
        //         self.0.as_bytes()
        //     }
        // }
        // impl Drop for MyString {
        //     fn drop(&mut self) {
        //         println!("drop {:?}", self.0);
        //         unsafe {
        //             ManuallyDrop::drop(&mut self.0);
        //         }
        //     }
        // }

        let x: usize = 0;
        let y = x.checked_add(1).unwrap();

        let env = Env::new();
        // let atom0 = env.make_atom("Ω'\x00".as_bytes());
        // let atom1 = env.make_atom_latin1("Ω'\x00".as_bytes());
        let atom0 = env.make_atom("'".as_bytes());
        let atom1 = env.make_atom_latin1("'".as_bytes());
        // let mut at = AtomTable::new();
        // at.intern_utf8(Bytes::from("hello").to_vec());
        // for i in 0..10 {
        //     // at.intern_utf8(MyString::new(format!("atom{}", i)).0.into_bytes());
        //     // at.intern_utf8(format!("Ωatom{}", i).into_bytes());
        //     at.intern_utf8(format!("atom{}", i).into_bytes());
        //     at.intern_latin1(format!("atom{}", i).into_bytes());
        // }
        // let atom0 = at.intern_utf8(&b"\xCE\xA9"[..]);
        // println!("at = {:?}", at);
        // println!("atom0 = {:?}", atom0);
        // let atom1 = at.intern_utf8("Ω".as_bytes());
        // let atom2 = at.intern_latin1(&b"\xCE\xA9"[..]);
        // println!("{:?}", std::str::from_utf8(at.get(atom0).unwrap()));
        // println!("atom0 == atom1 = {:?}", atom0 == atom1);
        // println!("atom0 == atom2 = {:?}", atom0 == atom2);
        // println!("atom1 == atom2 = {:?}", atom1 == atom2);
        // let v: Vec<u8> = at.get_entry(atom2).unwrap().iter_bytes().collect();
        // println!("atom2 = {:?}", v);
        // println!("atom2 utf8 = {:?}", std::str::from_utf8(&v[..]));
        // println!("at = {:?}", at);
        println!("env = {:?}", env);
        println!("atom0 = {:?}", atom0);
        println!("atom1 = {:?}", atom1);
        println!("atom0 == atom1 {:?}", atom0 == atom1);

        // let mut at = AtomTable::new();
        // let atom0 = at.intern_utf8(&b"\xCE\xA9"[..]);
        // println!("at = {:?}", at);
        // println!("atom0 = {:?}", atom0);
        // let atom1 = at.intern_utf8("Ω".as_bytes());
        // let atom2 = at.intern_latin1(&b"\xCE\xA9"[..]);
        // println!("{:?}", std::str::from_utf8(at.get(atom0).unwrap()));
        // println!("atom0 == atom1 = {:?}", atom0 == atom1);
        // println!("atom0 == atom2 = {:?}", atom0 == atom2);
        // println!("atom1 == atom2 = {:?}", atom1 == atom2);
        // let v: Vec<u8> = at.get_entry(atom2).unwrap().iter_bytes().collect();
        // println!("atom2 = {:?}", v);
        // println!("atom2 utf8 = {:?}", std::str::from_utf8(&v[..]));
        // println!("at = {:?}", at);

        // let mut cache = crate::dist::AtomCache::new();
        // println!("cache.get(0) = {:?}", cache.get(0));
        // cache.insert(0, std::rc::Rc::new(term::Atom::from("foo"))).unwrap();
        // println!("cache.get(0) = {:?}", cache.get(0));
        // cache.insert(2047, std::rc::Rc::new(term::Atom::from("bar"))).unwrap();

        // println!("\n\n{:?}", cache);

        // let atom0 = Term::from(term::Atom::from("atom0"));
        // let atom1 = Term::from(term::Atom::from("atom1"));
        // let atom2 = Term::from(term::Atom::from("atom2"));
        // let atom3 = Term::from(term::Atom::from("atom3"));
        // let atom4 = Term::from(term::Atom::from("atom4"));
        // let key = Term::from(term::Atom::from("key"));
        // let val = Term::from(term::Atom::from("val"));
        // let list = Term::from(term::List::from(vec![atom0, atom1, atom2, atom3, atom4]));
        // let map = Term::from(term::Map::from(vec![(key, val)]));
        // let term = Term::from(term::List::from(vec![list, map]));
        // // let term = Term::from(term::Atom::from("foo"));
        // println!("term = {:?}", term);
        // println!("term = {}", term);
        // println!("size = {:?}", SizeEncoder::encode(&term));
        // let result = 2 + 2;
        // assert_eq!(result, 4);
    }
}
