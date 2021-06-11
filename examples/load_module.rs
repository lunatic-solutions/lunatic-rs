use lunatic::process::Module;

fn main() {
    let module = include_bytes!("../target/wasm32-wasi/debug/examples/cli.wasm");
    let module = Module::load(module).unwrap();
    module.spawn("_start", 32).join().unwrap();
}
