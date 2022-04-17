use lunatic::{ProcessConfig, Task};
use lunatic_test::test;

#[test]
#[should_panic]
fn default_config_cant_spawn_sub_processes() {
    let config = ProcessConfig::new();
    let task = Task::spawn_link_config(&config, (), |_| {
        let sub_task = Task::spawn_link((), |_| {});
        sub_task.result();
    });
    task.result();
}

#[test]
fn config_with_spawn_permission() {
    let mut config = ProcessConfig::new();
    config.set_can_spawn_processes(true);

    let task = Task::spawn_link_config(&config, (), |_| {
        let sub_task = Task::spawn_link((), |_| {});
        sub_task.result();
    });
    task.result();

    assert_eq!(config.can_spawn_processes(), true);
}

#[test]
#[should_panic]
fn default_config_cant_create_configs() {
    let config = ProcessConfig::new();
    let task = Task::spawn_link_config(&config, (), |_| {
        ProcessConfig::new();
    });
    task.result();
}

#[test]
fn config_with_config_creation_permissions() {
    let mut config = ProcessConfig::new();
    config.set_can_create_configs(true);

    let task = Task::spawn_link_config(&config, (), |_| {
        ProcessConfig::new();
    });
    task.result();

    assert_eq!(config.can_create_configs(), true);
}

#[test]
#[should_panic]
fn config_with_memory_limit() {
    let mut config = ProcessConfig::new();
    config.set_max_memory(1_200_000); // ~ 1.2 Mb

    let task = Task::spawn_link_config(&config, (), |_| vec![0u64; 10_000]);
    task.result();
}

#[test]
#[should_panic]
fn config_with_compute_limit() {
    let mut config = ProcessConfig::new();
    config.set_max_fuel(1);

    let task = Task::spawn_link_config(&config, (), |_| (0..10_000).into_iter().count());
    task.result();
}

#[test]
fn config_env_variable() {
    let mut config = ProcessConfig::new();
    config.add_environment_variable("hello", "world");
    config.add_environment_variable("foo", "bar");

    let task = Task::spawn_link_config(&config, (), |_| {
        let hello = std::env::var("hello").unwrap();
        assert_eq!(hello, "world");
        let foo = std::env::var("foo").unwrap();
        assert_eq!(foo, "bar");
    });
    task.result();

    // The env var is not set in the parent
    assert!(std::env::var("hello").is_err());
    assert!(std::env::var("foo").is_err());
}

#[test]
fn config_cli_args() {
    let mut config = ProcessConfig::new();
    config.add_command_line_argument("test1");
    config.add_command_line_argument("test2");

    let task = Task::spawn_link_config(&config, (), |_| {
        let args: Vec<String> = std::env::args().collect();
        assert_eq!(args[0], "test1");
        assert_eq!(args[1], "test2");
    });
    task.result();
}
