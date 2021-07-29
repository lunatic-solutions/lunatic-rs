pub mod error {
    #[link(wasm_import_module = "lunatic::error")]
    extern "C" {
        pub fn string_size(error_id: u64) -> u32;
        pub fn to_string(error_id: u64, error_str: *mut u8);
        pub fn drop(error_id: u64);
    }
}

pub mod message {
    #[link(wasm_import_module = "lunatic::message")]
    extern "C" {
        pub fn create();
        pub fn set_buffer(data: *const u8, data_len: usize);
        pub fn add_process(process_id: u64) -> u64;
        pub fn add_tcp_stream(tcp_stream_id: u64) -> u64;
        pub fn send(process_id: u64) -> u32;
        pub fn prepare_receive(data_size: *mut usize, res_size: *mut usize) -> u32;
        pub fn receive(data: *mut u8, res: *mut u64);
    }
}

pub mod networking {
    #[link(wasm_import_module = "lunatic::networking")]
    extern "C" {
        pub fn resolve(name_str: *const u8, name_str_len: usize, id: *mut u64) -> u32;
        pub fn drop_dns_iterator(dns_iter_id: u64);
        pub fn resolve_next(
            dns_iter_id: u64,
            addr_type: *mut u32,
            addr: *mut u8,
            port: *mut u16,
            flow_info: *mut u32,
            scope_id: *mut u32,
        ) -> u32;
        pub fn tcp_bind(
            addr_type: u32,
            addr: *const u8,
            port: u32,
            flow_info: u32,
            scope_id: u32,
            id: *mut u64,
        ) -> u32;
        pub fn drop_tcp_listener(tcp_listener_id: u64);
        pub fn tcp_accept(listener_id: u64, id: *mut u64, dns_iter: *mut u64) -> u32;
        pub fn tcp_connect(
            addr_type: u32,
            addr: *const u8,
            port: u32,
            flow_info: u32,
            scope_id: u32,
            id: *mut u64,
        ) -> u32;
        pub fn drop_tcp_stream(tcp_stream_id: u64);
        pub fn clone_tcp_stream(tcp_stream_id: u64) -> u64;
        pub fn tcp_write_vectored(
            tcp_stream_id: u64,
            ciovec_array: *const u32,
            ciovec_array_len: usize,
            opaque: *mut u64,
        ) -> u32;
        pub fn tcp_read(
            tcp_stream_id: u64,
            buffer: *mut u8,
            buffer_len: usize,
            opaque: *mut u64,
        ) -> u32;
        pub fn tcp_flush(tcp_stream_id: u64, error_id: *mut u64) -> u32;
    }
}

pub mod process {
    #[link(wasm_import_module = "lunatic::process")]
    extern "C" {
        pub fn create_config(max_memory: u64, max_fuel: u64) -> u64;
        pub fn drop_config(config_id: u64);
        pub fn allow_namespace(config_id: u64, name_str: *const u8, name_str_len: usize);
        pub fn add_plugin(
            config_id: u64,
            plugin_data: *const u8,
            plugin_data_len: usize,
            id: *mut u64,
        ) -> u32;
        pub fn create_environment(config_id: u64, id: *mut u64) -> u32;
        pub fn drop_environment(env_id: u64);
        pub fn add_module(
            env_id: u64,
            module_data: *const u8,
            module_data_len: usize,
            id: *mut u64,
        ) -> u32;
        pub fn add_this_module(env_id: u64, id: *mut u64) -> u32;
        pub fn drop_module(mod_id: u64);
        pub fn spawn(
            link: u32,
            module_id: u64,
            function_str: *const u8,
            function_str_len: usize,
            params: *const u8,
            params_len: usize,
            id: *mut u64,
        ) -> u32;
        pub fn inherit_spawn(
            link: u32,
            function_str: *const u8,
            function_str_len: usize,
            params: *const u8,
            params_len: usize,
            id: *mut u64,
        ) -> u32;
        pub fn drop_process(process_id: u64);
        pub fn clone_process(tcp_stream_id: u64) -> u64;
        pub fn sleep_ms(millis: u64);
        pub fn die_when_link_dies(trap: u32);
        pub fn this() -> u64;
        pub fn join(process_id: u64) -> u32;
    }
}
