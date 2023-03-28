use std::collections::HashMap;
// Lunatic with metrics enabled is required (enabled by default)
// To collect the metrics, prometheus feature should be also enabled
// and lunatic has to be run with --prometheus flag
use std::time::Duration;

use lunatic::metrics::{
    add_event, counter, decrement_gauge, histogram, increment_counter, increment_gauge, Span,
};
use lunatic::{sleep, Mailbox};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let span = Span::new("myspan", &()).unwrap();
    let mut attributes = HashMap::new();
    attributes.insert("target", "MyModule");
    attributes.insert("message", "Hello world!");
    span.add_event("my_event", &attributes).unwrap();

    let inner_span = Span::new_with_parent(&span, "my_inner_span", &()).unwrap();
    inner_span.add_event("my_inner_event", &()).unwrap();

    let inner_span_b = Span::new_with_parent(&span, "my_inner_span_b", &()).unwrap();
    inner_span_b.add_event("my_inner_event_b", &()).unwrap();

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
