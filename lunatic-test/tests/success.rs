use lunatic_test::test;

#[test]
fn success_test() {
    assert!(true);
}

mod sub_module {
    use lunatic_test::test;

    #[test]
    fn success_test() {
        assert_eq!(1, 1, "One and one should be equal");
    }

    mod sub_sub_module {
        #[lunatic_test::test]
        fn success_test() {
            // Empty test
        }
    }
}
