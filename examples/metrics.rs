use std::collections::HashMap;
use std::fmt;
// Lunatic with metrics enabled is required (enabled by default)
// To collect the metrics, prometheus feature should be also enabled
// and lunatic has to be run with --prometheus flag
use std::time::Duration;

use log::Level;
use lunatic::metrics::{
    add_event, counter, decrement_gauge, histogram, increment_counter, increment_gauge, Span,
};
use lunatic::{info, info_span, sleep, Mailbox};
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

    // let span = Span::new("myspan", &()).unwrap();
    // let mut attributes = HashMap::new();
    // attributes.insert("target", "MyModule");
    // attributes.insert("message", "Hello world!");
    // span.add_event("my_event", &attributes).unwrap();
    //
    // let inner_span = Span::new_with_parent(&span, "my_inner_span", &()).unwrap();
    // inner_span.add_event("my_inner_event", &()).unwrap();
    //
    // let inner_span_b = Span::new_with_parent(&span, "my_inner_span_b", &()).unwrap();
    // inner_span_b.add_event("my_inner_event_b", &()).unwrap();
    //
    // let attributes = lunatic::valueset!(entry: "foo" = 1, "hey");
    // dbg!(attributes);
    //
    // let attrs = lunatic::attributes!(target: "trgt", Level::Info, "foo" = 1, "hey");
    // dbg!(attrs);

    // let span_id = unsafe {
    //     let name = "my_span";
    //     let name_len = name.len();
    //     lunatic::host::api::metrics::start_span(name.as_ptr(), name_len, &[] as *const u8, 0)
    // };
    // unsafe {
    //     let name = "my_event";
    //     let name_len = name.len();
    //     lunatic::host::api::metrics::add_event(
    //         span_id,
    //         name.as_ptr(),
    //         name_len,
    //         &[] as *const u8,
    //         0,
    //     );
    // }
    // lunatic::spawn!(|mailbox: Mailbox<()>| {
    //     unsafe {
    //         let name = "my_event";
    //         let name_len = name.len();
    //         lunatic::host::api::metrics::add_event(
    //             span_id,
    //             name.as_ptr(),
    //             name_len,
    //             &[] as *const u8,
    //             0,
    //         );
    //     }
    // });
    // lunatic::sleep(Duration::from_secs(100));
    // panic!("Nope!");
    // unsafe {
    //     lunatic::host::api::metrics::drop_span(span_id);
    // }
    // dbg!(span_id);

    // counter("lunatic::metrics_example::counter", 42);
    // for i in 0..6000 {
    //     increment_counter("lunatic::metrics_example::counter");
    //     if i % 50 < 25 {
    //         increment_gauge("lunatic::metrics_example::gauge", 1.0);
    //     } else {
    //         decrement_gauge("lunatic::metrics_example::gauge", 1.0);
    //     }
    //     histogram("lunatic::metrics_example::histogram", i as f64 % 50.0);
    //     sleep(Duration::from_millis(10));
    // }
}
