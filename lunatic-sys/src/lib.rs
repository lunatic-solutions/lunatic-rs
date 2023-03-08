#[export_name = "lunatic_alloc"]
pub extern "C" fn lunatic_alloc(len: u32) -> *mut u8 {
    let buf = Vec::with_capacity(len as usize);
    let mut buf = std::mem::ManuallyDrop::new(buf);
    buf.as_mut_ptr()
}
