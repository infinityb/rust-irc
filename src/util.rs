#[deriving(PartialEq, Copy, Clone, Show)]
pub struct StringSlicer {
    from_idx: uint,
    to_idx: uint
}

impl StringSlicer {
    #[inline]
    pub fn new(from_idx: uint, to_idx: uint) -> StringSlicer {
        StringSlicer {
            from_idx: from_idx,
            to_idx: to_idx
        }
    }

    #[inline]
    pub fn slice_on<'a>(&self, string: &'a str) -> &'a str {
        string[self.from_idx..self.to_idx]
    }
}

#[deriving(PartialEq, Copy, Clone, Show)]
pub struct OptionalStringSlicer {
    exists: bool,
    from_idx: uint,
    to_idx: uint
}

impl OptionalStringSlicer {
    #[inline]
    pub fn new_some(from_idx: uint, to_idx: uint) -> OptionalStringSlicer {
        OptionalStringSlicer {
            exists: true,
            from_idx: from_idx,
            to_idx: to_idx
        }
    }

    #[inline]
    pub fn new_none() -> OptionalStringSlicer {
        OptionalStringSlicer {
            exists: false,
            from_idx: 0,
            to_idx: 0
        }
    }

    #[inline]
    pub fn slice_on<'a>(&self, string: &'a str) -> Option<&'a str> {
        if self.exists {
            Some(string[self.from_idx..self.to_idx])
        } else {
            None
        }
    }
}