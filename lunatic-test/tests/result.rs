use lunatic_test::test;

#[test]
fn success() -> Result<(), ()> {
    Ok(())
}

#[test]
#[ignore]
fn failure() -> Result<(), ()> {
    Err(())
}
