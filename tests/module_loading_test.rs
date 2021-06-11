use lunatic::process::Module;

#[test]
fn load_module_test() {
    let wasm = include_bytes!("wasm/add.wasm");
    Module::load(wasm).unwrap();
}

#[test]
fn run_module_with_import() {
    let import = include_bytes!("wasm/add_import.wasm");
    let module_import = Module::load(import).unwrap();

    let wasm = include_bytes!("wasm/add.wasm");
    let module = Module::load(wasm).unwrap();
    module
        .spawn_with("test", 32, &[("env".to_string(), module_import)])
        .join()
        .unwrap();
}

#[test]
fn allow_proxy_yield() {
    let import = include_bytes!("wasm/proxy_import_allow.wasm");
    let module_import = Module::load(import).unwrap();

    let wasm = include_bytes!("wasm/proxy.wasm");
    let module = Module::load(wasm).unwrap();
    module
        .spawn_with("test", 32, &[("lunatic".to_string(), module_import)])
        .join()
        .unwrap();
}

#[test]
fn forbid_proxy_yield() {
    let import = include_bytes!("wasm/proxy_import_forbid.wasm");
    let module_import = Module::load(import).unwrap();

    let wasm = include_bytes!("wasm/proxy.wasm");
    let module = Module::load(wasm).unwrap();
    let result = module
        .spawn_with("test", 32, &[("lunatic".to_string(), module_import)])
        .join();
    assert_eq!(result, Err(()));
}
