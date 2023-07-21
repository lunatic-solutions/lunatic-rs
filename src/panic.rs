use crate::host;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Panicked;

/// Invokes a closure, capturing a panic if one occurs.
///
/// This function will return Ok with the closure’s result if the closure does
/// not panic, and will return `Err(Panicked)` if the closure panics.
///
/// Different than [`catch_unwind`](std::panic::catch_unwind), this function
/// doesn't depend on unwinding to work. This allows it to work in lunatic
/// (WebAssembly). This also means that resources acquired inside of the closure
/// will not be freed on panic.
///
/// Because of this, it is **not** recommended to use this function for a
/// general try/catch mechanism. The `Result` type is more appropriate to use
/// for functions that can fail on a regular basis.
pub fn catch_panic<R, F: FnOnce() -> R>(f: F) -> Result<R, Panicked> {
    let function = Box::new(f);
    let raw_function = Box::<F>::into_raw(function) as usize;
    let raw_result =
        unsafe { host::api::trap::catch(re_entry::<R, F> as usize, raw_function) } as *mut R;
    if raw_result.is_null() {
        Err(Panicked)
    } else {
        Ok(*unsafe { Box::<R>::from_raw(raw_result) })
    }
}

/// Wrapper function to help transfer the generic types R & F through the host.
fn re_entry<R, F: FnOnce() -> R>(pointer: usize) -> usize {
    let function = unsafe { Box::<F>::from_raw(pointer as *mut F) };
    let result = function();
    let result = Box::new(result);
    Box::<R>::into_raw(result) as usize
}
