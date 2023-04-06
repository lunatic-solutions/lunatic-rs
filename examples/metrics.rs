use std::fmt;

use lunatic::metrics::Meter;
use lunatic::{info, info_span, Mailbox};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Person {
    name: String,
    age: u8,
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} is {}", self.name, self.age)
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    // Start a span of work
    let span = info_span!("my_span");

    // Any type which implements Serialize can be used in the log macros
    let null = ();
    let bool_t = true;
    let bool_f = false;
    let number = 10.43;
    let string = "Hello, World!";
    let array = vec![1, 2, 3];
    let object = Person {
        name: "John Doe".to_string(),
        age: 23,
    };

    info!(
        null,
        bool_t,
        bool_f,
        number,
        string,
        array,
        object,
        "Additional log message, with {}!", // The last argument is a message, and is optional
        "formatting"                        // It supports the same arguments as format_args!
    );

    // The % prefix can be used to format a variable using its fmt::Display implementation
    info!(%object, "formatted object");

    // The ? prefix can be used to format a variable using its fmt::Debug implementation
    info!(?object, "debug object");

    // The name of an attribute can be defined with `name = value`
    info!(person = object, "person object");

    // The target can be set manually, but defaults to the module_path!
    info!(target: "my_app", "a log from my_app");

    // The parent span can also be set manually, but uses the last created span by default
    info!(parent: span, "a log under my_span");

    // Meters are used to create counters and histograms
    let meter = Meter::new("my-meter");

    // Counters can only increment up, and should not be added with a negative value
    let counter = meter.counter("my-counter").build();
    counter.add(5).done();

    // Up-down counters can increment up and down
    let up_down_counter = meter.up_down_counter("my-up-down-counter").build();
    up_down_counter.add(5).done();
    up_down_counter.add(-5).done();

    // Histograms records a distribution of values
    let histogram = meter.histogram("my-histogram").build();
    histogram.record(5).done();
    histogram.record(10).done();
}
