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


    /// Composes slicers. The new slices will function as if
    /// self.slice_from(ss.slice_on(...)) was called, carrying any
    /// Options over
    #[inline]
    pub fn slice_from<'a>(&self, ss: &StringSlicer) -> StringSlicer {
        let slicer = StringSlicer::new(
            ss.from_idx + self.from_idx,
            ss.from_idx + self.to_idx);
        if slicer.to_idx > ss.to_idx {
            panic!("excessively large subslice");
        }
        slicer
    }

    /// Composes slicers. The new slices will function as if
    /// self.slice_from(ss.slice_on(...)) was called, carrying any
    /// Options over
    #[inline]
    pub fn slice_from_opt<'a>(&self, ss: &OptionalStringSlicer) -> OptionalStringSlicer {
        let mut slicer = OptionalStringSlicer {
            exists: ss.exists,
            from_idx: ss.from_idx + self.from_idx,
            to_idx: ss.from_idx + self.to_idx,
        };
        if !slicer.exists {
            slicer.from_idx = 0;
            slicer.to_idx = 0;
        }
        if slicer.to_idx > ss.to_idx {
            panic!("excessively large subslice");
        }
        slicer
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

    /// Composes slicers. The new slices will function as if
    /// self.slice_from(ss.slice_on(...)) was called, carrying any
    /// Options over
    #[inline]
    pub fn slice_from<'a>(&self, ss: &StringSlicer) -> OptionalStringSlicer {
        let mut slicer = OptionalStringSlicer {
            exists: self.exists,
            from_idx: ss.from_idx + self.from_idx,
            to_idx: ss.from_idx + self.to_idx,
        };
        if !slicer.exists {
            slicer.from_idx = 0;
            slicer.to_idx = 0;
        }
        if slicer.to_idx > ss.to_idx {
            panic!("excessively large subslice");
        }
        slicer
    }

    /// Composes slicers. The new slices will function as if
    /// self.slice_from(ss.slice_on(...)) was called, carrying any
    /// Options over
    #[inline]
    pub fn slice_from_opt<'a>(&self, ss: &OptionalStringSlicer) -> OptionalStringSlicer {
        let mut slicer = OptionalStringSlicer {
            exists: self.exists && ss.exists,
            from_idx: ss.from_idx + self.from_idx,
            to_idx: ss.from_idx + self.to_idx,
        };
        if !slicer.exists {
            slicer.from_idx = 0;
            slicer.to_idx = 0;
        }
        if slicer.to_idx > ss.to_idx {
            panic!("excessively large subslice");
        }
        slicer
    }
}