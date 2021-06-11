#[link(wasm_import_module = "lunatic")]
extern "C" {
    fn yield_();
}

#[export_name = "yield_"]
pub extern "C" fn yield__() {
    unsafe { yield_() };
}
