// TODO: Move out into separate crate (lunatic-bindings?) & auto generate from lunatic's source?

pub(crate) mod error {
    #[link(wasm_import_module = "lunatic::error")]
    extern "C" {
        pub(crate) fn string_size(error_id: u64) -> u32;
        pub(crate) fn to_string(error_id: u64, error_str: *mut u8);
        pub(crate) fn drop(error_id: u64);
    }
}

pub(crate) mod message {
    #[link(wasm_import_module = "lunatic::message")]
    extern "C" {
        pub(crate) fn create_data(tag: i64, capacity: u64);
        pub(crate) fn write_data(data: *const u8, data_len: usize) -> usize;
        pub(crate) fn read_data(data: *mut u8, data_len: usize) -> usize;
        #[allow(dead_code)]
        pub(crate) fn seek_data(position: u64);
        pub(crate) fn get_tag() -> i64;
        #[allow(dead_code)]
        pub(crate) fn data_size() -> u64;
        pub(crate) fn push_process(process_id: u64) -> u64;
        pub(crate) fn take_process(index: u64) -> u64;
        pub(crate) fn push_tcp_stream(tcp_stream_id: u64) -> u64;
        pub(crate) fn take_tcp_stream(index: u64) -> u64;
        pub(crate) fn send(process_id: u64);
        pub(crate) fn send_receive_skip_search(process_id: u64, timeout: u32) -> u32;
        pub(crate) fn receive(tag: *const i64, tag_len: usize, timeout: u32) -> u32;
    }
}

pub(crate) mod networking {
    #[link(wasm_import_module = "lunatic::networking")]
    extern "C" {
        pub(crate) fn resolve(
            name_str: *const u8,
            name_str_len: usize,
            timeout: u32,
            id: *mut u64,
        ) -> u32;
        pub(crate) fn drop_dns_iterator(dns_iter_id: u64);
        pub(crate) fn resolve_next(
            dns_iter_id: u64,
            addr_type: *mut u32,
            addr: *mut u8,
            port: *mut u16,
            flow_info: *mut u32,
            scope_id: *mut u32,
        ) -> u32;
        pub(crate) fn tcp_bind(
            addr_type: u32,
            addr: *const u8,
            port: u32,
            flow_info: u32,
            scope_id: u32,
            id: *mut u64,
        ) -> u32;
        pub(crate) fn drop_tcp_listener(tcp_listener_id: u64);
        pub(crate) fn tcp_local_addr(tcp_listener_id: u64, addr_dns_iter: *mut u64) -> u32;
        pub(crate) fn tcp_accept(listener_id: u64, id: *mut u64, peer_dns_iter: *mut u64) -> u32;
        pub(crate) fn tcp_connect(
            addr_type: u32,
            addr: *const u8,
            port: u32,
            flow_info: u32,
            scope_id: u32,
            timeout: u32,
            id: *mut u64,
        ) -> u32;
        pub(crate) fn drop_tcp_stream(tcp_stream_id: u64);
        pub(crate) fn clone_tcp_stream(tcp_stream_id: u64) -> u64;
        pub(crate) fn tcp_write_vectored(
            tcp_stream_id: u64,
            ciovec_array: *const u32,
            ciovec_array_len: usize,
            timeout: u32,
            opaque: *mut u64,
        ) -> u32;
        pub(crate) fn tcp_read(
            tcp_stream_id: u64,
            buffer: *mut u8,
            buffer_len: usize,
            timeout: u32,
            opaque: *mut u64,
        ) -> u32;
        pub(crate) fn tcp_flush(tcp_stream_id: u64, error_id: *mut u64) -> u32;
    }
}

pub(crate) mod process {
    #[link(wasm_import_module = "lunatic::process")]
    extern "C" {
        pub(crate) fn compile_module(data: *const u8, data_len: usize, id: *mut u64) -> i32;
        pub(crate) fn drop_module(config_id: u64);
        pub(crate) fn create_config() -> u64;
        pub(crate) fn drop_config(config_id: u64);
        pub(crate) fn config_set_max_memory(config_id: u64, max_memory: u64);
        pub(crate) fn config_get_max_memory(config_id: u64) -> u64;
        pub(crate) fn config_set_max_fuel(config_id: u64, max_fuel: u64);
        pub(crate) fn config_get_max_fuel(config_id: u64) -> u64;
        pub(crate) fn config_can_compile_modules(config_id: u64) -> u32;
        pub(crate) fn config_set_can_compile_modules(config_id: u64, can: u32);
        pub(crate) fn config_can_create_configs(config_id: u64) -> u32;
        pub(crate) fn config_set_can_create_configs(config_id: u64, can: u32);
        pub(crate) fn config_can_spawn_processes(config_id: u64) -> u32;
        pub(crate) fn config_set_can_spawn_processes(config_id: u64, can: u32);
        pub(crate) fn spawn(
            link: i64,
            config_id: i64,
            module_id: i64,
            function: *const u8,
            function_len: usize,
            params: *const u8,
            params_len: usize,
            id: *mut u64,
        ) -> u32;
        pub(crate) fn drop_process(process_id: u64);
        pub(crate) fn clone_process(process_id: u64) -> u64;
        pub(crate) fn sleep_ms(millis: u64);
        pub(crate) fn die_when_link_dies(trap: u32);
        pub(crate) fn this() -> u64;
        pub(crate) fn id(process_id: u64, uuid: *mut [u8; 16]);
        pub(crate) fn link(tag: i64, process_id: u64);
        pub(crate) fn unlink(process_id: u64);
    }
}

pub(crate) mod wasi {
    #[link(wasm_import_module = "lunatic::wasi")]
    extern "C" {
        pub(crate) fn config_add_environment_variable(
            config_id: u64,
            key: *const u8,
            key_len: usize,
            value: *const u8,
            value_len: usize,
        );
        pub(crate) fn config_add_command_line_argument(
            config_id: u64,
            key: *const u8,
            key_len: usize,
        );
        pub(crate) fn config_preopen_dir(config_id: u64, key: *const u8, key_len: usize);
    }
}
