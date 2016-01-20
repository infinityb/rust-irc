use std::ops;
use std::fmt;

pub struct Slice {
    pub inner: [u8]
}

impl fmt::Debug for Slice {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.inner.fmt(formatter)
    }
}

impl Slice {
    pub fn to_owned(&self) -> Vec<u8> {
        self.inner.to_vec()
    }
}

impl ops::Deref for Slice {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.inner
    }
}
