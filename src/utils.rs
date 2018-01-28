// transfer ref to mut ref
#[cfg_attr(feature = "cargo-clippy", allow(mut_from_ref))]
#[inline]
pub unsafe fn transmute_mut<T: ?Sized>(r: &T) -> &mut T {
    &mut *(r as *const T as *mut T)
}
