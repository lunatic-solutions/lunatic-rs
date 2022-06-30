use lunatic_test::test;

#[test]
#[should_panic(expected = "contained")]
fn panic_contains_expected_string() {
    panic!("Word contained is present");
}

#[test]
#[should_panic]
fn panics() {
    panic!("Any string will match");
}

#[test]
#[should_panic]
fn assert_failure_is_also_a_panic() {
    assert!(false);
}

#[test]
#[should_panic(expected = "#")]
fn hashtag_works_in_panic_string() {
    panic!("#")
}
