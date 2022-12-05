# Supervised Abstract Process Example

This example shows how to call a supervised abstract process across different files using the auto-generated public traits from the `abstract_process` macro.

## Files
* `counter_abstract_process.rs`
    * Main abstract process definition.
* `main.rs`
    * Supervisor definition.
    * Example usage of cross-module `AbstractProcess` defined through the `abstract_process` macro.
