use lunatic::panic::catch_panic;
use lunatic_test::test;

#[test]
fn catch_panic_simple() {
    assert!(catch_panic(|| {}).is_ok());
    assert!(catch_panic(|| panic!()).is_err());
}

#[test]
fn catch_panic_capture() {
    let hello = String::from("Hello");
    let result = catch_panic(|| hello).unwrap();
    assert_eq!(result, "Hello");
}

#[test]
fn catch_assert_fail() {
    assert!(catch_panic(|| assert!(false)).is_err())
}
