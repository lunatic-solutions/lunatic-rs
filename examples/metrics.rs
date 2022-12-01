// Lunatic with metrics enabled is required (enabled by default)
// To collect the metrics, prometheus feature should be also enabled
// and lunatic has to be run with --prometheus flag
use std::time::Duration;

use lunatic::metrics::{counter, decrement_gauge, histogram, increment_counter, increment_gauge};
use lunatic::{sleep, Mailbox};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    counter("lunatic::metrics_example::counter", 42);
    for i in 0..6000 {
        increment_counter("lunatic::metrics_example::counter");
        if i % 50 < 25 {
            increment_gauge("lunatic::metrics_example::gauge", 1.0);
        } else {
            decrement_gauge("lunatic::metrics_example::gauge", 1.0);
        }
        histogram("lunatic::metrics_example::histogram", i as f64 % 50.0);
        sleep(Duration::from_millis(10));
    }
}
