use lunatic_test::test;

#[test]
#[ignore]
fn assert_failed() {
    assert_eq!(1, 2);
}

#[test]
#[ignore]
#[should_panic]
fn panic_failed() {
    // Didn't panic
}
