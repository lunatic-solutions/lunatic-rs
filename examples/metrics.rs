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
    // let span = info_span!("my_span");

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
    // {
    //     (|message: std::fmt::Arguments<'_>| {
    //         let name = "event_:0";
    //         let attrs = lunatic::metrics::Attributes::new(
    //             ("module::path"),
    //             (lunatic::metrics::Level::Info),
    //             message,
    //             "",
    //             0,
    //             0,
    //             "module::path",
    //             [].into_iter()
    //                 .collect::<std::collections::BTreeMap<&'static str, serde_json::Value>>(),
    //         );
    //         lunatic::metrics::add_event(None, name, Some(&attrs));
    //     })(format_args!("Hello {}!", bool_f))
    // }

    info!(
        null,
        bool_t,
        bool_f,
        number,
        string,
        array,
        object,
        "Additional log message, with {}!",
        "formatting"
    );

    // {
    //     let name = "event_:0";
    //     let attributes = {
    //         let (message, attributes) = {
    //             let mut message: Option<std::fmt::Arguments<'_>> = None;
    //             let attributes: std::collections::BTreeMap<&'static str, serde_json::Value> = {
    //                 message = Some(format_args!("Hello, world"));
    //                 [].into_iter().collect()
    //             };
    //             (message, attributes)
    //         };
    //         lunatic::metrics::Attributes::new(
    //             ("module::path"),
    //             (lunatic::metrics::Level::Info),
    //             message,
    //             "",
    //             0,
    //             "module::path",
    //             attributes,
    //         )
    //     };
    //     lunatic::metrics::add_event(None, name, &attributes);
    // }

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
