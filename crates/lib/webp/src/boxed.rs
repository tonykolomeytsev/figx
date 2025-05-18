//! Safe RAII wrappers for `WebPFree`.
#![allow(unused)]

use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::os::raw::*;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::ptr::NonNull;
use std::slice;

use crate::error::WebPSimpleError;

/// A safe RAII wrapper for `WebPFree`.
///
/// `WebpBox` is much like `Box`, except what function is used for freeing.
///
/// ## Example
///
/// The main usecase is as a return value from libwebp.
///
/// ```rust
/// use libwebp::WebPEncodeRGBA;
///
/// let buf: &[u8];
/// # let buf: &[u8] = &[
/// #     255, 255, 255, 255,
/// #     255, 0, 0, 255,
/// #     0, 255, 0, 255,
/// #     0, 0, 255, 255,
/// # ];
/// // This variable has the type of `WebpBox<[u8]>`.
/// let data = WebPEncodeRGBA(buf, 2, 2, 8, 75.0).unwrap();
/// ```
pub struct WebpBox<T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

unsafe impl<T: ?Sized + Send> Send for WebpBox<T> {}
unsafe impl<T: ?Sized + Sync> Sync for WebpBox<T> {}
impl<T: ?Sized + UnwindSafe> UnwindSafe for WebpBox<T> {}
impl<T: ?Sized + RefUnwindSafe> RefUnwindSafe for WebpBox<T> {}

impl<T: ?Sized> WebpBox<T> {
    /// Creates `WebpBox` from a raw pointer.
    ///
    /// ## Safety
    ///
    /// - `raw` must be non-null.
    /// - `raw` must be well-aligned.
    /// - The pointee must be valid as `T`.
    /// - The pointee must be exclusively accessible.
    /// - `raw` must be freeable via `WebPFree`.
    pub unsafe fn from_raw(raw: *mut T) -> WebpBox<T> {
        Self {
            ptr: unsafe { NonNull::new_unchecked(raw) },
            _marker: PhantomData,
        }
    }

    /// Turns `WebpBox` into a raw pointer without freeing anything.
    pub fn into_raw(b: WebpBox<T>) -> *mut T {
        let ptr = b.ptr;
        mem::forget(b);
        ptr.as_ptr()
    }
}

impl<T: ?Sized> Deref for WebpBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for WebpBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> Drop for WebpBox<T> {
    fn drop(&mut self) {
        unsafe {
            WebPFree(self.ptr.as_ptr() as *mut c_void);
        }
    }
}

use libwebp_sys::WebPFree;

impl<T: fmt::Debug + ?Sized> fmt::Debug for WebpBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self as &T, f)
    }
}

#[inline]
pub(crate) unsafe fn wrap_bytes<F>(
    ptr: *mut u8,
    get_len: F,
) -> Result<WebpBox<[u8]>, WebPSimpleError>
where
    F: FnOnce() -> usize,
{
    if !ptr.is_null() {
        let len = get_len();
        Ok(unsafe { WebpBox::from_raw(slice::from_raw_parts_mut(ptr, len)) })
    } else {
        Err(WebPSimpleError)
    }
}

/// A variant of `WebpBox` for return values from `WebPDecodeYUV`.
///
/// See [`WebPDecodeYUV`] for examples.
///
/// [`WebPDecodeYUV`]: ../fn.WebPDecodeYUV.html
pub struct WebpYuvBox {
    y: NonNull<[u8]>,
    u: NonNull<[u8]>,
    v: NonNull<[u8]>,
}

unsafe impl Send for WebpYuvBox {}
unsafe impl Sync for WebpYuvBox {}

impl WebpYuvBox {
    /// Creates `WebpYuvBox` from raw pointers.
    ///
    /// ## Safety
    ///
    /// - `y`, `u`, `v` must be non-null.
    /// - The pointees must be valid as `[u8]`.
    /// - The pointees must be exclusively accessible.
    /// - The head pointer of `y` must be freeable via `WebPFree`.
    /// - The pointees of `u` and `v` must be within the allocated area
    ///   designated by the head pointer of `y`.
    pub unsafe fn from_raw_yuv(y: *mut [u8], u: *mut [u8], v: *mut [u8]) -> WebpYuvBox {
        unsafe {
            Self {
                y: NonNull::new_unchecked(y),
                u: NonNull::new_unchecked(u),
                v: NonNull::new_unchecked(v),
            }
        }
    }

    /// Turns `WebpYuvBox` into raw pointers without freeing anything.
    pub fn into_raw_yuv(self) -> (*mut [u8], *mut [u8], *mut [u8]) {
        let y = self.y.as_ptr();
        let u = self.u.as_ptr();
        let v = self.v.as_ptr();
        mem::forget(self);
        (y, u, v)
    }

    /// Immutably deferences to the `y` slice.
    pub fn y(&self) -> &[u8] {
        unsafe { self.y.as_ref() }
    }
    /// Mutably deferences to the `y` slice.
    pub fn y_mut(&mut self) -> &mut [u8] {
        unsafe { self.y.as_mut() }
    }

    /// Immutably deferences to the `u` slice.
    pub fn u(&self) -> &[u8] {
        unsafe { self.u.as_ref() }
    }
    /// Mutably deferences to the `u` slice.
    pub fn u_mut(&mut self) -> &mut [u8] {
        unsafe { self.u.as_mut() }
    }

    /// Immutably deferences to the `v` slice.
    pub fn v(&self) -> &[u8] {
        unsafe { self.v.as_ref() }
    }
    /// Mutably deferences to the `v` slice.
    pub fn v_mut(&mut self) -> &mut [u8] {
        unsafe { self.v.as_mut() }
    }

    /// Immutably, simultaneously dereferences to the `y`, `u`, and `v` slices.
    pub fn yuv(&self) -> (&[u8], &[u8], &[u8]) {
        let y = unsafe { self.y.as_ref() };
        let u = unsafe { self.u.as_ref() };
        let v = unsafe { self.v.as_ref() };
        (y, u, v)
    }
    /// Mutably, simultaneously dereferences to the `y`, `u`, and `v` slices.
    pub fn yuv_mut(&mut self) -> (&mut [u8], &mut [u8], &mut [u8]) {
        let y = unsafe { self.y.as_mut() };
        let u = unsafe { self.u.as_mut() };
        let v = unsafe { self.v.as_mut() };
        (y, u, v)
    }

    /// Turns into a `y` pointer, discarding `u`, `v` slices.
    pub fn into_y(self) -> WebpBox<[u8]> {
        let y = self.y;
        mem::forget(self);
        WebpBox {
            ptr: y,
            _marker: PhantomData,
        }
    }
}

impl Drop for WebpYuvBox {
    fn drop(&mut self) {
        unsafe {
            WebPFree(self.y.as_ptr() as *mut c_void);
        }
    }
}

impl fmt::Debug for WebpYuvBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebpYuvBox")
            .field("y", &self.y())
            .field("u", &self.u())
            .field("v", &self.v())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::panic::{RefUnwindSafe, UnwindSafe};

    #[allow(unused)]
    fn test_auto_traits() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        fn is_unwind_safe<T: UnwindSafe>() {}
        fn is_ref_unwind_safe<T: RefUnwindSafe>() {}

        is_send::<WebpBox<[u8]>>();
        is_sync::<WebpBox<[u8]>>();
        is_unwind_safe::<WebpBox<[u8]>>();
        is_ref_unwind_safe::<WebpBox<[u8]>>();

        is_send::<WebpYuvBox>();
        is_sync::<WebpYuvBox>();
        is_unwind_safe::<WebpYuvBox>();
        is_ref_unwind_safe::<WebpYuvBox>();
    }
}
