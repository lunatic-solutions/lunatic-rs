use lunatic::{spawn_link, ProcessConfig};
use lunatic_test::test;

#[test]
#[should_panic]
fn default_config_cant_spawn_sub_processes() {
    let config = ProcessConfig::new().unwrap();
    let task = spawn_link!(@task &config, || {
        let sub_task = spawn_link!(@task || {});
        let _ =sub_task.result();
    });
    let _ = task.result();
}

#[test]
fn config_with_spawn_permission() {
    let mut config = ProcessConfig::new().unwrap();
    config.set_can_spawn_processes(true);

    let task = spawn_link!(@task &config,  || {
        let sub_task = spawn_link!(@task || {});
        let _ =sub_task.result();
    });
    let _ = task.result();

    assert_eq!(config.can_spawn_processes(), true);
}

#[test]
#[should_panic]
fn default_config_cant_create_configs() {
    let config = ProcessConfig::new().unwrap();
    let task = spawn_link!(@task &config, || {
        ProcessConfig::new().unwrap();
    });
    let _ = task.result();
}

#[test]
fn config_with_config_creation_permissions() {
    let mut config = ProcessConfig::new().unwrap();
    config.set_can_create_configs(true);

    let task = spawn_link!(@task &config, || {
        ProcessConfig::new().unwrap();
    });
    let _ = task.result();

    assert_eq!(config.can_create_configs(), true);
}

#[test]
fn config_without_config_creation_permissions() {
    let config = ProcessConfig::new().unwrap();

    let task = spawn_link!(@task &config, || {
        ProcessConfig::new().is_err()
    });

    assert_eq!(task.result(), true);
    assert_eq!(config.can_create_configs(), false);
}

#[test]
#[should_panic]
fn config_with_memory_limit() {
    let mut config = ProcessConfig::new().unwrap();
    config.set_max_memory(1_200_000); // ~ 1.2 Mb

    let task = spawn_link!(@task&config, || vec![0u64; 10_000]);
    let _ = task.result();
}

#[test]
#[should_panic]
fn config_with_compute_limit() {
    let mut config = ProcessConfig::new().unwrap();
    config.set_max_fuel(1);

    let task =
        spawn_link!(@task &config, || (0..10_000).into_iter().map(|v| v.to_string()).count());
    let _ = task.result();
}

#[test]
fn config_env_variable() {
    let mut config = ProcessConfig::new().unwrap();
    config.add_environment_variable("hello", "world");
    config.add_environment_variable("foo", "bar");

    let task = spawn_link!(@task &config, || {
        let hello = std::env::var("hello").unwrap();
        assert_eq!(hello, "world");
        let foo = std::env::var("foo").unwrap();
        assert_eq!(foo, "bar");
    });
    let _ = task.result();

    // The env var is not set in the parent
    assert!(std::env::var("hello").is_err());
    assert!(std::env::var("foo").is_err());
}

#[test]
fn config_cli_args() {
    let mut config = ProcessConfig::new().unwrap();
    config.add_command_line_argument("test1");
    config.add_command_line_argument("test2");

    let task = spawn_link!(@task &config, || {
        let args: Vec<String> = std::env::args().collect();
        assert_eq!(args[0], "test1");
        assert_eq!(args[1], "test2");
    });
    let _ = task.result();
}
