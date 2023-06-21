use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Nil;
impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[]")
    }
}
impl From<()> for Nil {
    fn from(_x: ()) -> Self {
        Nil
    }
}
