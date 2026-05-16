use core::ffi::c_void;
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NSWindowHandle(NonNull<c_void>);

impl NSWindowHandle {
    /// Wraps a caller-owned `NSWindow *` for `StoreKit` APIs that require a window.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a live `NSWindow` for the duration of any `StoreKit` call that
    /// borrows the returned handle. The handle does not retain the window and must not be
    /// used after the underlying window has been deallocated.
    pub unsafe fn from_raw(ptr: *mut c_void) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub const fn as_raw(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}
