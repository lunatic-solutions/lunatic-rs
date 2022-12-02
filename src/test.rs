// This function is used internally by the `#[lunatic::test]` macro to check if
// the value returned from the test is not `Result::Err`.
pub fn assert_test_result<T: TestReturnValue + std::fmt::Debug>(result: T) {
    assert!(
        result.is_success(),
        "the test returned a value indicating failure ({:?})",
        result
    );
}

pub trait TestReturnValue {
    fn is_success(&self) -> bool;
}

impl TestReturnValue for () {
    fn is_success(&self) -> bool {
        true
    }
}

impl<T, E> TestReturnValue for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }
}
