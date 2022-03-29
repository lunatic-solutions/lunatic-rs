use lunatic_test::test;

#[test]
#[should_panic(expected = "marko")]
fn panics() {
    println!("Hello world");
    panic!("mark polo");
}

#[test]
fn another_failure() {
    assert_eq!(1, 1, "1 != 0");
}

#[test]
#[should_panic]
fn didnt_panic() {}
