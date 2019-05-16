use core::mem;
use core::pin::Pin;
use core::ptr::null_mut;

use failure::Error;
use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};

use crate::common::{Database, Streaming};
use crate::errors::AsResult;
use crate::ffi;
use crate::runtime::{MatchEventCallback, ScratchRef};

impl Database<Streaming> {
    /// Provides the size of the stream state allocated by a single stream opened against the given database.
    pub fn stream_size(&self) -> Result<usize, Error> {
        let mut size: usize = 0;

        unsafe { ffi::hs_stream_size(self.as_ptr(), &mut size).map(|_| size) }
    }
}

impl Database<Streaming> {
    /// Open and initialise a stream.
    pub fn open_stream(&self) -> Result<Stream, Error> {
        let mut s = null_mut();

        unsafe { ffi::hs_open_stream(self.as_ptr(), 0, &mut s).map(|_| Stream::from_ptr(s)) }
    }
}

foreign_type! {
    /// A pattern matching state can be maintained across multiple blocks of target data
    pub type Stream {
        type CType = ffi::hs_stream_t;

        fn drop = drop_stream;
        fn clone = clone_stream;
    }
}

fn drop_stream(_s: *mut ffi::hs_stream_t) {}

unsafe fn clone_stream(s: *mut ffi::hs_stream_t) -> *mut ffi::hs_stream_t {
    let mut p = null_mut();

    ffi::hs_copy_stream(&mut p, s).expect("copy stream");

    p
}

impl StreamRef {
    /// Reset a stream to an initial state.
    pub fn reset<'a, D>(
        &self,
        scratch: &ScratchRef,
        callback: MatchEventCallback<'a, D>,
        context: Option<Pin<&'a mut D>>,
    ) -> Result<(), Error> {
        unsafe {
            ffi::hs_reset_stream(
                self.as_ptr(),
                0,
                scratch.as_ptr(),
                mem::transmute(callback),
                mem::transmute(context),
            )
            .ok()
        }
    }
}

impl Stream {
    /// Close a stream.
    pub fn close<'a, D>(
        self,
        scratch: &ScratchRef,
        callback: MatchEventCallback<'a, D>,
        context: Option<Pin<&'a mut D>>,
    ) -> Result<(), Error> {
        unsafe {
            ffi::hs_close_stream(
                self.as_ptr(),
                scratch.as_ptr(),
                mem::transmute(callback),
                mem::transmute(context),
            )
            .ok()
        }
    }
}
