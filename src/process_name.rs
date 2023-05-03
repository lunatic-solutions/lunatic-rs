/// ProcessName is used to uniquely name a process from a Rust type.
///
/// It is implemented for common string types such as `&str` and `String`,
/// but can be implemented on a custom type to create a process name type.
///
/// It is best used with the [ProcessName](lunatic_macros::ProcessName) derive macro
/// to generate a unique name.
///
/// # Example
///
/// ```
/// #[derive(ProcessName)]
/// struct LoggingProcess;
///
/// let process = lunatic::spawn!(|| { /* ... */});
/// process.register(&LoggingProcess);
/// ```
///
/// Alternatively, you can implement the trait manually.
///
/// ```
/// struct LoggingProcess;
///
/// impl ProcessName for LoggingProcess {
///     fn process_name(&self) -> &str {
///         "global_logging_process"
///     }
/// }
/// ```
///
/// For more information, see the [derive macro](lunatic_macros::ProcessName) docs.
pub trait ProcessName {
    fn process_name(&self) -> &str;
}

impl ProcessName for str {
    fn process_name(&self) -> &str {
        self
    }
}

impl ProcessName for &str {
    fn process_name(&self) -> &str {
        self
    }
}

impl ProcessName for String {
    fn process_name(&self) -> &str {
        self.as_str()
    }
}

impl<'a> ProcessName for std::borrow::Cow<'a, str> {
    fn process_name(&self) -> &str {
        self
    }
}
