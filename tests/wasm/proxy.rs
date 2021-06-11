#[link(wasm_import_module = "lunatic")]
extern "C" {
    fn yield_();
}

#[export_name = "test"]
pub extern "C" fn test() {
    unsafe { yield_() };
}
