// transfer ref to mut ref
// we can use RefCell<T> here for the reade/writer
// but this unsafe way is a bit faster
#[cfg_attr(feature = "cargo-clippy", allow(mut_from_ref))]
#[inline]
pub unsafe fn transmute_mut<T: ?Sized>(r: &T) -> &mut T {
    &mut *(r as *const T as *mut T)
}
