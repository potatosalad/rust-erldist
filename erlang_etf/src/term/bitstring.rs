use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Bitstring {
    pub data: Vec<u8>,
    pub bits: u8,
}
impl Bitstring {
    pub fn is_binary(&self) -> bool {
        if self.bits % 8 == 0 {
            true
        } else {
            false
        }
    }

    pub fn is_bit_binary(&self) -> bool {
        if self.bits % 8 != 0 {
            true
        } else {
            false
        }
    }
}
impl fmt::Display for Bitstring {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<<")?;
        for (i, b) in self.data.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            if i == self.data.len() - 1 && self.is_bit_binary() {
                write!(f, "{}:{}", b, self.bits % 8)?;
            } else {
                write!(f, "{}", b)?;
            }
        }
        write!(f, ">>")?;
        Ok(())
    }
}
impl<'a> From<&'a [u8]> for Bitstring {
    fn from(data: &'a [u8]) -> Self {
        Bitstring {
            data: Vec::from(data),
            bits: 0,
        }
    }
}
impl From<Vec<u8>> for Bitstring {
    fn from(data: Vec<u8>) -> Self {
        Bitstring { data, bits: 0 }
    }
}
impl From<(Vec<u8>, u8)> for Bitstring {
    fn from((data, bits): (Vec<u8>, u8)) -> Self {
        Bitstring {
            data,
            bits: bits % 8,
        }
    }
}
