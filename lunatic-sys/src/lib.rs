#[export_name = "lunatic_alloc"]
pub extern "C" fn lunatic_alloc(len: u32) -> *mut u8 {
    let buf = Vec::with_capacity(len as usize);
    let mut buf = std::mem::ManuallyDrop::new(buf);
    buf.as_mut_ptr()
}

/// This export is used by the host as a trampoline to re-enter the Wasm
/// instance. Every re-entrance can be used as a point to catch a Wasm trap.
#[export_name = "_lunatic_catch_trap"]
extern "C" fn lunatic_catch_trap(function: usize, pointer: usize) -> usize {
    let function: fn(usize) -> usize = unsafe { std::mem::transmute(function) };
    function(pointer)
}
