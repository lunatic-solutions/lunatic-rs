//! Lunatic VM host functions.

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
        pub fn create_data(tag: i64, capacity: u64);
        pub fn write_data(data: *const u8, data_len: usize) -> usize;
        pub fn read_data(data: *mut u8, data_len: usize) -> usize;
        #[allow(dead_code)]
        pub fn seek_data(position: u64);
        pub fn get_tag() -> i64;
        #[allow(dead_code)]
        pub fn data_size() -> u64;
        pub fn push_tcp_stream(tcp_stream_id: u64) -> u64;
        pub fn take_tcp_stream(index: u64) -> u64;
        pub fn send(process_id: u64);
        pub fn send_receive_skip_search(process_id: u64, timeout: u64) -> u32;
        pub fn receive(tag: *const i64, tag_len: usize, timeout: u64) -> u32;
    }
}

pub mod timer {
    #[link(wasm_import_module = "lunatic::timer")]
    extern "C" {
        pub fn send_after(process_id: u64, duration: u64) -> u64;
        pub fn cancel_timer(timer_id: u64) -> u32;
    }
}

pub mod networking {
    #[link(wasm_import_module = "lunatic::networking")]
    extern "C" {
        pub fn resolve(name_str: *const u8, name_str_len: usize, timeout: u64, id: *mut u64)
            -> u32;
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
        pub fn udp_bind(
            addr_type: u32,
            addr: *const u8,
            port: u32,
            flow_info: u32,
            scope_id: u32,
            id: *mut u64,
        ) -> u32;
        pub fn drop_tcp_listener(tcp_listener_id: u64);
        pub fn drop_udp_socket(udp_socket_id: u64);
        pub fn tcp_local_addr(tcp_listener_id: u64, addr_dns_iter: *mut u64) -> u32;
        pub fn udp_local_addr(udp_socket_id: u64, addr_dns_iter: *mut u64) -> u32;
        pub fn tcp_accept(listener_id: u64, id: *mut u64, peer_dns_iter: *mut u64) -> u32;
        pub fn tcp_connect(
            addr_type: u32,
            addr: *const u8,
            port: u32,
            flow_info: u32,
            scope_id: u32,
            timeout: u64,
            id: *mut u64,
        ) -> u32;
        pub fn udp_connect(
            udp_socket_id: u64,
            addr_type: u32,
            addr: *const u8,
            port: u32,
            flow_info: u32,
            scope_id: u32,
            timeout: u64,
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
        pub fn tcp_peek(
            tcp_stream_id: u64,
            buffer: *mut u8,
            buffer_len: usize,

            opaque: *mut u64,
        ) -> u32;
        pub fn udp_send(
            udp_socket_id: u64,
            buffer: *const u8,
            buffer_len: usize,

            opaque: *mut u64,
        ) -> u32;
        pub fn udp_send_to(
            udp_socket_id: u64,
            buffer: *const u8,
            buffer_len: usize,
            addr_type: u32,
            addr: *const u8,
            port: u32,
            flow_info: u32,
            scope_id: u32,
            opaque: *mut u64,
        ) -> u32;
        pub fn udp_receive(
            udp_socket_id: u64,
            buffer: *mut u8,
            buffer_len: usize,
            opaque: *mut u64,
        ) -> u32;
        pub fn udp_receive_from(
            udp_socket_id: u64,
            buffer: *mut u8,
            buffer_len: usize,
            opaque: *mut u64,
            dns_iter_ptr: *mut u64,
        ) -> u32;
        pub fn set_udp_socket_ttl(udp_socket_id: u64, ttl: u32);
        pub fn set_udp_socket_broadcast(udp_socket_id: u64, broadcast: u32);
        pub fn get_udp_socket_ttl(udp_socket_id: u64) -> u32;
        pub fn get_udp_socket_broadcast(udp_socket_id: u64) -> i32;
        pub fn clone_udp_socket(udp_socket_id: u64) -> u64;
        pub fn tcp_flush(tcp_stream_id: u64, error_id: *mut u64) -> u32;
        pub fn set_read_timeout(tcp_stream_id: u64, duration: u64) -> u32;
        pub fn get_read_timeout(tcp_stream_id: u64) -> u64;
        pub fn set_write_timeout(tcp_stream_id: u64, duration: u64) -> u32;
        pub fn get_write_timeout(tcp_stream_id: u64) -> u64;
        pub fn set_peek_timeout(tcp_stream_id: u64, duration: u64) -> u32;
        pub fn get_peek_timeout(tcp_stream_id: u64) -> u64;
    }
}

pub mod process {
    #[link(wasm_import_module = "lunatic::process")]
    extern "C" {
        pub fn compile_module(data: *const u8, data_len: usize, id: *mut u64) -> i32;
        pub fn drop_module(config_id: u64);
        pub fn create_config() -> u64;
        pub fn drop_config(config_id: u64);
        pub fn config_set_max_memory(config_id: u64, max_memory: u64);
        pub fn config_get_max_memory(config_id: u64) -> u64;
        pub fn config_set_max_fuel(config_id: u64, max_fuel: u64);
        pub fn config_get_max_fuel(config_id: u64) -> u64;
        pub fn config_can_compile_modules(config_id: u64) -> u32;
        pub fn config_set_can_compile_modules(config_id: u64, can: u32);
        pub fn config_can_create_configs(config_id: u64) -> u32;
        pub fn config_set_can_create_configs(config_id: u64, can: u32);
        pub fn config_can_spawn_processes(config_id: u64) -> u32;
        pub fn config_set_can_spawn_processes(config_id: u64, can: u32);
        pub fn spawn(
            link: i64,
            config_id: i64,
            module_id: i64,
            function: *const u8,
            function_len: usize,
            params: *const u8,
            params_len: usize,
            id: *mut u64,
        ) -> u32;
        pub fn sleep_ms(millis: u64);
        pub fn die_when_link_dies(trap: u32);
        pub fn process_id() -> u64;
        pub fn link(tag: i64, process_id: u64);
        pub fn unlink(process_id: u64);
        pub fn kill(process_id: u64);
    }
}

pub mod registry {
    #[link(wasm_import_module = "lunatic::registry")]
    extern "C" {
        pub fn put(name: *const u8, name_len: usize, node_id: u64, process_id: u64);
        pub fn get(
            name: *const u8,
            name_len: usize,
            node_id: *mut u64,
            process_id: *mut u64,
        ) -> u32;
        pub fn remove(name: *const u8, name_len: usize);
    }
}

pub mod wasi {
    #[link(wasm_import_module = "lunatic::wasi")]
    extern "C" {
        pub fn config_add_environment_variable(
            config_id: u64,
            key: *const u8,
            key_len: usize,
            value: *const u8,
            value_len: usize,
        );
        pub fn config_add_command_line_argument(config_id: u64, key: *const u8, key_len: usize);
        pub fn config_preopen_dir(config_id: u64, key: *const u8, key_len: usize);
    }
}

pub mod distributed {
    #[link(wasm_import_module = "lunatic::distributed")]
    extern "C" {
        pub fn get_nodes(nodes_ptr: *mut u64, nodes_len: u32) -> u32;
        pub fn exec_lookup_nodes(
            query_ptr: *const u8,
            query_len: u32,
            query_id_ptr: *mut u64,
            node_len_ptr: *mut u32,
        ) -> u32;
        pub fn copy_lookup_nodes_results(query_id: u64, nodes_ptr: *mut u64, nodes_len: u32)
            -> i32;
        pub fn nodes_count() -> u32;
        pub fn node_id() -> u64;
        pub fn module_id() -> u64;
        pub fn send(node_id: u64, process_id: u64);
        pub fn send_receive_skip_search(node_id: u64, process_id: u64, timeout: u64) -> u32;
        pub fn spawn(
            node_id: u64,
            config_id: i64,
            module_id: u64,
            function: *const u8,
            function_len: usize,
            params: *const u8,
            params_len: usize,
            id: *mut u64,
        ) -> u32;
    }
}

pub mod version {
    #[link(wasm_import_module = "lunatic::version")]
    extern "C" {
        pub fn major() -> u32;
        pub fn minor() -> u32;
        pub fn patch() -> u32;
    }
}
