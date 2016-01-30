use std::ops;
use std::marker;
use std::slice;
use std::mem;

use ::slice::Slice;
use super::IrcMsg;


pub struct MessageGroupBuf {
    inner: Vec<u8>,
}

impl MessageGroupBuf {
    pub fn new() -> MessageGroupBuf {
        MessageGroupBuf {
            inner: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> MessageGroupBuf {
        MessageGroupBuf {
            inner: Vec::with_capacity(cap),
        }
    }

    pub fn push(&mut self, msg: &IrcMsg) {
        let body = msg.as_bytes();
        self.inner.reserve_exact(body.len() + 1);
        self.inner.extend_from_slice(body);
        self.inner.push(0);
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

impl ops::Deref for MessageGroupBuf {
    type Target = MessageGroup;

    fn deref<'a>(&'a self) -> &'a MessageGroup {
        unsafe { MessageGroup::from_u8_slice_unchecked(&self.inner) }
    }
}

pub struct MessageGroup {
    inner: Slice,
}

impl MessageGroup {
    /// The following function allows unchecked construction of a message
    /// group from a u8 slice.  This is unsafe because it does not maintain
    /// the MessageGroup invariant.
    unsafe fn from_u8_slice_unchecked(s: &[u8]) -> &MessageGroup {
        mem::transmute(s)
    }

    /// The following function allows unchecked construction of a mutable
    /// message group from a mutable u8 slice.  This is unsafe because it
    /// does not maintain the MessageGroup invariant.
    unsafe fn from_u8_slice_unchecked_mut(s: &mut [u8]) -> &mut MessageGroup {
        mem::transmute(s)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn iter(&self) -> Iter {
        Iter::new(self.as_bytes())
    }
}

pub struct Iter<'a> {
    ptr: *const u8,
    len: usize,
    _marker: marker::PhantomData<&'a u8>,
}

impl<'a> Iter<'a> {
    pub fn new(slice: &[u8]) -> Iter {
        Iter {
            ptr: slice.as_ptr(),
            len: slice.len(),
            _marker: marker::PhantomData,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a IrcMsg;

    #[inline]
    fn next(&mut self) -> Option<&'a IrcMsg> {
        unsafe {
            let start_at = self.ptr;
            let mut data_len: Option<usize> = None;
            for idx in 0..self.len {
                if *self.ptr.offset(idx as isize) == 0 {
                    data_len = Some(idx as usize + 1);
                    break;
                }
            }

            if data_len.is_none() {
                return None;
            }

            let data_len = data_len.unwrap();

            self.len -= data_len;
            self.ptr = self.ptr.offset(data_len as isize);
            let msg_data = slice::from_raw_parts(start_at, data_len - 1);
            Some(IrcMsg::from_u8_slice_unchecked(msg_data))
        }
    }
}