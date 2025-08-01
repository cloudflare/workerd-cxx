#![cfg(feature = "alloc")]
#![allow(missing_docs)]

use crate::exception::repr;
use crate::exception::Exception;
use alloc::string::String;
use alloc::string::ToString;
use core::ptr::{self, NonNull};
use core::result::Result as StdResult;

#[repr(C)]
pub struct Result {
    err: *mut repr::KjException,
}

impl Result {
    pub(crate) fn ok() -> Self {
        Result {
            err: ptr::null_mut(),
        }
    }

    pub(crate) fn error<E: ToKjException>(err: E, file: &str, line: u32) -> Self {
        let err = err.to_kj_exception();
        let msg = err.description();

        let err = unsafe {
            repr::new(
                msg.as_ptr(),
                msg.len(),
                file.as_ptr(),
                file.len(),
                line.try_into().unwrap_or_default(),
            )
        };
        Self { err }
    }
}

pub unsafe fn r#try<T, E>(ret: *mut T, result: StdResult<T, E>, file: &str, line: u32) -> Result
where
    E: ToKjException,
{
    match result {
        Ok(ok) => {
            unsafe { ptr::write(ret, ok) }
            Result::ok()
        }
        Err(err) => Result::error(err.to_kj_exception(), file, line),
    }
}

impl Result {
    pub unsafe fn exception(self) -> StdResult<(), Exception> {
        let err = self.err;
        core::mem::forget(self);
        match NonNull::new(err) {
            Some(err) => Err(Exception { err }),
            None => Ok(()),
        }
    }
}

impl Drop for Result {
    fn drop(&mut self) {
        unsafe { repr::drop_in_place(self.err) }
    }
}

pub struct KjException {
    description: String,
}

impl KjException {
    pub fn new(description: String) -> Self {
        Self { description }
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

impl core::fmt::Debug for KjException {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KjException").field("description", &self.description).finish()
    }
}

pub trait ToKjException {
    fn to_kj_exception(self) -> KjException;
}

impl ToKjException for std::io::Error {
    fn to_kj_exception(self) -> KjException {
        KjException::new(self.to_string())
    }
}

impl ToKjException for alloc::string::String {
    fn to_kj_exception(self) -> KjException {
        KjException::new(self)
    }
}


impl ToKjException for KjException {
    fn to_kj_exception(self) -> KjException {
        self
    }
}
