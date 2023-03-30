/// Constructs a new span.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: crate#using-the-macros
///
/// # Examples
///
/// Creating a new span:
/// ```
/// # use tracing::{span, Level};
/// # fn main() {
/// let span = span!(Level::Trace, "my span");
/// let _enter = span.enter();
/// // do work inside the span...
/// # }
/// ```
#[macro_export]
macro_rules! span {
    (target: $target:expr, parent: $parent:expr, $lvl:expr, $name:expr) => {
        $crate::span!(target: $target, parent: $parent, $lvl, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
        {
            $crate::metrics::Span::new_with_parent(
                $parent,
                $name,
                Some(&$crate::valueset!(target: $target, $lvl, $($fields)*))
            )
                .expect("attributes should not fail to serialize")
        }
    };
    (target: $target:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
        {
            $crate::metrics::Span::new(
                $name,
                Some(&$crate::valueset!(target: $target, $lvl, $($fields)*))
            )
                .expect("attributes should not fail to serialize")
        }
    };
    (target: $target:expr, parent: $parent:expr, $lvl:expr, $name:expr) => {
        $crate::span!(target: $target, parent: $parent, $lvl, $name,)
    };
    (parent: $parent:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            $name,
            $($fields)*
        )
    };
    (parent: $parent:expr, $lvl:expr, $name:expr) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            $name,
        )
    };
    (target: $target:expr, $lvl:expr, $name:expr, $($fields:tt)*) => {
        $crate::span!(
            target: $target,
            $lvl,
            $name,
            $($fields)*
        )
    };
    (target: $target:expr, $lvl:expr, $name:expr) => {
        $crate::span!(target: $target, $lvl, $name,)
    };
    ($lvl:expr, $name:expr, $($fields:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $lvl,
            $name,
            $($fields)*
        )
    };
    ($lvl:expr, $name:expr) => {
        $crate::span!(
            target: module_path!(),
            $lvl,
            $name,
        )
    };
}

/// Constructs a span at the trace level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: crate#using-the-macros
/// [attributes]: crate#configuring-attributes
/// [Fields]: crate#recording-fields
/// [`span!`]: span!
///
/// # Examples
///
/// ```rust
/// # use tracing::{trace_span, span, Level};
/// # fn main() {
/// trace_span!("my_span");
/// // is equivalent to:
/// span!(Level::Trace, "my_span");
/// # }
/// ```
///
/// ```rust
/// # use tracing::{trace_span, span, Level};
/// # fn main() {
/// let span = trace_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! trace_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::metrics::Level::Trace,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::trace_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::trace_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::metrics::Level::Trace,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::trace_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            $name,
            $($field)*
        )
    };
    ($name:expr) => { $crate::trace_span!($name,) };
}

/// Constructs a span at the debug level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: crate#using-the-macros
/// [attributes]: crate#configuring-attributes
/// [Fields]: crate#recording-fields
/// [`span!`]: span!
///
/// # Examples
///
/// ```rust
/// # use tracing::{debug_span, span, Level};
/// # fn main() {
/// debug_span!("my_span");
/// // is equivalent to:
/// span!(Level::Debug, "my_span");
/// # }
/// ```
///
/// ```rust
/// # use tracing::debug_span;
/// # fn main() {
/// let span = debug_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! debug_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::metrics::Level::Debug,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::debug_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::debug_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::metrics::Level::Debug,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::debug_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {$crate::debug_span!($name,)};
}

/// Constructs a span at the info level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: crate#using-the-macros
/// [attributes]: crate#configuring-attributes
/// [Fields]: crate#recording-fields
/// [`span!`]: span!
///
/// # Examples
///
/// ```rust
/// # use tracing::{span, info_span, Level};
/// # fn main() {
/// info_span!("my_span");
/// // is equivalent to:
/// span!(Level::Info, "my_span");
/// # }
/// ```
///
/// ```rust
/// # use tracing::info_span;
/// # fn main() {
/// let span = info_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! info_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::metrics::Level::Info,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::info_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::info_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::metrics::Level::Info,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::info_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {$crate::info_span!($name,)};
}

/// Constructs a span at the warn level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: crate#using-the-macros
/// [attributes]: crate#configuring-attributes
/// [Fields]: crate#recording-fields
/// [`span!`]: span!
///
/// # Examples
///
/// ```rust
/// # use tracing::{warn_span, span, Level};
/// # fn main() {
/// warn_span!("my_span");
/// // is equivalent to:
/// span!(Level::Warn, "my_span");
/// # }
/// ```
///
/// ```rust
/// use tracing::warn_span;
/// # fn main() {
/// let span = warn_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! warn_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::metrics::Level::Warn,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::warn_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::warn_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::metrics::Level::Warn,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::warn_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {$crate::warn_span!($name,)};
}
/// Constructs a span at the error level.
///
/// [Fields] and [attributes] are set using the same syntax as the [`span!`]
/// macro.
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: crate#using-the-macros
/// [attributes]: crate#configuring-attributes
/// [Fields]: crate#recording-fields
/// [`span!`]: span!
///
/// # Examples
///
/// ```rust
/// # use tracing::{span, error_span, Level};
/// # fn main() {
/// error_span!("my_span");
/// // is equivalent to:
/// span!(Level::Error, "my_span");
/// # }
/// ```
///
/// ```rust
/// # use tracing::error_span;
/// # fn main() {
/// let span = error_span!("my span");
/// span.in_scope(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! error_span {
    (target: $target:expr, parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            parent: $parent,
            $crate::metrics::Level::Error,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::error_span!(target: $target, parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::error_span!(parent: $parent, $name,)
    };
    (target: $target:expr, $name:expr, $($field:tt)*) => {
        $crate::span!(
            target: $target,
            $crate::metrics::Level::Error,
            $name,
            $($field)*
        )
    };
    (target: $target:expr, $name:expr) => {
        $crate::error_span!(target: $target, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        $crate::span!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {$crate::error_span!($name,)};
}

/// Constructs a new `Event`.
///
/// The event macro is invoked with a `Level` and up to 32 key-value fields.
/// Optionally, a format string and arguments may follow the fields; this will
/// be used to construct an implicit field named "message".
///
/// See [the top-level documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [lib]: crate#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::{event, Level};
///
/// # fn main() {
/// let data = (42, "forty-two");
/// let private_data = "private";
/// let error = "a bad error";
///
/// event!(Level::Error, %error, "Received error");
/// event!(
///     target: "app_events",
///     Level::Warn,
///     private_data,
///     ?data,
///     "App warning: {}",
///     error
/// );
/// event!(Level::Info, the_answer = data.0);
/// # }
/// ```
///
// /// Note that *unlike `span!`*, `event!` requires a value for all fields. As
// /// events are recorded immediately when the macro is invoked, there is no
// /// opportunity for fields to be recorded later. A trailing comma on the final
// /// field is valid.
// ///
// /// For example, the following does not compile:
// /// ```rust,compile_fail
// /// # use tracing::{Level, event};
// /// # fn main() {
// /// event!(Level::Info, foo = 5, bad_field, bar = "hello")
// /// #}
// /// ```
#[macro_export]
macro_rules! event {
    (target: $target:expr, parent: $parent:expr, $lvl:expr, { $($fields:tt)* } )=> ({
        let name = concat!(
            "event_",
            file!(),
            ":",
            line!()
        );
        $parent.add_event(name, Some(&$crate::valueset!(target: $target, $lvl, $($fields)*)))
            .expect("attributes should not fail to serialize");
    });

    (target: $target:expr, parent: $parent:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => (
        $crate::event!(
            target: $target,
            parent: $parent,
            $lvl,
            { $($fields),* $($arg)+ }
        )
    );
    (target: $target:expr, parent: $parent:expr, $lvl:expr, $($k:ident).+ = $($fields:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $lvl, { $($k).+ = $($fields)* })
    );
    (target: $target:expr, parent: $parent:expr, $lvl:expr, $($arg:tt)+) => (
        $crate::event!(target: $target, parent: $parent, $lvl, { $($arg)+ })
    );
    (target: $target:expr, $lvl:expr, { $($fields:tt)* } )=> ({
        let name = concat!(
            "event_",
            file!(),
            ":",
            line!()
        );
        $crate::metrics::add_event(None, name, Some(&$crate::valueset!(target: $target, $lvl, $($fields)*)))
            .expect("attributes should not fail to serialize");
    });
    (target: $target:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => (
        $crate::event!(
            target: $target,
            $lvl,
            { $($fields),* $($arg)+ }
        )
    );
    (target: $target:expr, $lvl:expr, $($k:ident).+ = $($fields:tt)* ) => (
        $crate::event!(target: $target, $lvl, { $($k).+ = $($fields)* })
    );
    (target: $target:expr, $lvl:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $lvl, { $($arg)+ })
    );
    (parent: $parent:expr, $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            { $($fields),* $($arg)+ }
        )
    );
    (parent: $parent:expr, $lvl:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $lvl:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $lvl:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $lvl:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $lvl:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $lvl:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $lvl,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $lvl:expr, $($arg:tt)+ ) => (
        $crate::event!(target: module_path!(), parent: $parent, $lvl, { $($arg)+ })
    );
    ( $lvl:expr, { $($fields:tt)* }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $lvl,
            { $($fields),* $($arg)+ }
        )
    );
    ($lvl:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $lvl,
            { $($k).+ = $($field)*}
        )
    );
    ($lvl:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $lvl,
            { $($k).+, $($field)*}
        )
    );
    ($lvl:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $lvl,
            { ?$($k).+, $($field)*}
        )
    );
    ($lvl:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $lvl,
            { %$($k).+, $($field)*}
        )
    );
    ($lvl:expr, ?$($k:ident).+) => (
        $crate::event!($lvl, ?$($k).+,)
    );
    ($lvl:expr, %$($k:ident).+) => (
        $crate::event!($lvl, %$($k).+,)
    );
    ($lvl:expr, $($k:ident).+) => (
        $crate::event!($lvl, $($k).+,)
    );
    ( $lvl:expr, $($arg:tt)+ ) => (
        $crate::event!(target: module_path!(), $lvl, { $($arg)+ })
    );
}

/// Constructs an event at the trace level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: event!
/// [lib]: crate#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::trace;
/// # #[derive(Debug, Copy, Clone)] struct Position { x: f32, y: f32 }
/// # impl Position {
/// # const ORIGIN: Self = Self { x: 0.0, y: 0.0 };
/// # fn dist(&self, other: Position) -> f32 {
/// #    let x = (other.x - self.x).exp2(); let y = (self.y - other.y).exp2();
/// #    (x + y).sqrt()
/// # }
/// # }
/// # fn main() {
/// let pos = Position { x: 3.234, y: -1.223 };
/// let origin_dist = pos.dist(Position::ORIGIN);
///
/// trace!(position = ?pos, ?origin_dist);
/// trace!(
///     target: "app_events",
///     position = ?pos,
///     "x is {} and y is {}",
///     if pos.x >= 0.0 { "positive" } else { "negative" },
///     if pos.y >= 0.0 { "positive" } else { "negative" }
/// );
/// # }
/// ```
#[macro_export]
macro_rules! trace {
    (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Trace, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Trace, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Trace, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Trace, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Trace, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Trace,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Trace, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Trace, { $($k).+ $($field)* })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Trace, { ?$($k).+ $($field)* })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Trace, { %$($k).+ $($field)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Trace, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            { $($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Trace,
            {},
            $($arg)+
        )
    );
}

/// Constructs an event at the debug level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: event!
/// [lib]: crate#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::debug;
/// # fn main() {
/// # #[derive(Debug)] struct Position { x: f32, y: f32 }
///
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!(?pos.x, ?pos.y);
/// debug!(target: "app_events", position = ?pos, "New position");
/// # }
/// ```
#[macro_export]
macro_rules! debug {
    (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Debug, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Debug, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Debug, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Debug, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Debug, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Debug,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Debug, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Debug, { $($k).+ $($field)* })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Debug, { ?$($k).+ $($field)* })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Debug, { %$($k).+ $($field)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Debug, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { $($k).+ = $($field)*}
        )
    );
    (?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { ?$($k).+ = $($field)*}
        )
    );
    (%$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { %$($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Debug,
            {},
            $($arg)+
        )
    );
}

/// Constructs an event at the info level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: event!
/// [lib]: crate#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::info;
/// # // this is so the test will still work in no-std mode
/// # #[derive(Debug)]
/// # pub struct Ipv4Addr;
/// # impl Ipv4Addr { fn new(o1: u8, o2: u8, o3: u8, o4: u8) -> Self { Self } }
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// use tracing::field;
///
/// let addr = Ipv4Addr::new(127, 0, 0, 1);
/// let conn = Connection { port: 40, speed: 3.20 };
///
/// info!(conn.port, "connected to {:?}", addr);
/// info!(
///     target: "connection_events",
///     ip = ?addr,
///     conn.port,
///     ?conn.speed,
/// );
/// # }
/// ```
#[macro_export]
macro_rules! info {
     (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Info, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Info, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Info, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Info, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Info, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Info,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Info, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Info, { $($k).+ $($field)* })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Info, { ?$($k).+ $($field)* })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Info, { $($k).+ $($field)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Info, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { $($k).+ = $($field)*}
        )
    );
    (?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { ?$($k).+ = $($field)*}
        )
    );
    (%$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { %$($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Info,
            {},
            $($arg)+
        )
    );
}

/// Constructs an event at the warn level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: event!
/// [lib]: crate#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::warn;
/// # fn main() {
///
/// let warn_description = "Invalid Input";
/// let input = &[0x27, 0x45];
///
/// warn!(?input, warning = warn_description);
/// warn!(
///     target: "input_events",
///     warning = warn_description,
///     "Received warning for input: {:?}", input,
/// );
/// # }
/// ```
#[macro_export]
macro_rules! warn {
     (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Warn, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Warn, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Warn, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Warn, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Warn, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Warn,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Warn, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Warn, { $($k).+ $($field)* })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Warn, { ?$($k).+ $($field)* })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Warn, { %$($k).+ $($field)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Warn, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { $($k).+ = $($field)*}
        )
    );
    (?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { ?$($k).+ = $($field)*}
        )
    );
    (%$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { %$($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Warn,
            {},
            $($arg)+
        )
    );
}

/// Constructs an event at the error level.
///
/// This functions similarly to the [`event!`] macro. See [the top-level
/// documentation][lib] for details on the syntax accepted by
/// this macro.
///
/// [`event!`]: event!
/// [lib]: crate#using-the-macros
///
/// # Examples
///
/// ```rust
/// use tracing::error;
/// # fn main() {
///
/// let (err_info, port) = ("No connection", 22);
///
/// error!(port, error = %err_info);
/// error!(target: "app_events", "App Error: {}", err_info);
/// error!({ info = err_info }, "error on port: {}", port);
/// # }
/// ```
#[macro_export]
macro_rules! error {
     (target: $target:expr, parent: $parent:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Error, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Error, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, ?$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Error, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, %$($k:ident).+ $($field:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Error, { $($k).+ $($field)+ })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, parent: $parent, $crate::metrics::Level::Error, {}, $($arg)+)
    );
    (parent: $parent:expr, { $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            { $($field)+ },
            $($arg)+
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            { $($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            { ?$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            { %$($k).+ = $($field)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            { $($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            { ?$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            { %$($k).+, $($field)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            parent: $parent,
            $crate::metrics::Level::Error,
            {},
            $($arg)+
        )
    );
    (target: $target:expr, { $($field:tt)* }, $($arg:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Error, { $($field)* }, $($arg)*)
    );
    (target: $target:expr, $($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Error, { $($k).+ $($field)* })
    );
    (target: $target:expr, ?$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Error, { ?$($k).+ $($field)* })
    );
    (target: $target:expr, %$($k:ident).+ $($field:tt)* ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Error, { %$($k).+ $($field)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::event!(target: $target, $crate::metrics::Level::Error, {}, $($arg)+)
    );
    ({ $($field:tt)+ }, $($arg:tt)+ ) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { $($field)+ },
            $($arg)+
        )
    );
    ($($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { $($k).+ = $($field)*}
        )
    );
    (?$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { ?$($k).+ = $($field)*}
        )
    );
    (%$($k:ident).+ = $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { %$($k).+ = $($field)*}
        )
    );
    ($($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { $($k).+, $($field)*}
        )
    );
    (?$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { ?$($k).+, $($field)*}
        )
    );
    (%$($k:ident).+, $($field:tt)*) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { %$($k).+, $($field)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { ?$($k).+ }
        )
    );
    (%$($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { %$($k).+ }
        )
    );
    ($($k:ident).+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            { $($k).+ }
        )
    );
    ($($arg:tt)+) => (
        $crate::event!(
            target: module_path!(),
            $crate::metrics::Level::Error,
            {},
            $($arg)+
        )
    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! valueset {

    // === base case ===
    (@ { $(,)* $($val:expr),* $(,)* }, $target:expr, $lvl:expr, $message:ident, $next:expr $(,)*) => {{
        // [ $($val),* ].into_iter().collect()
        let attributes: std::collections::BTreeMap<&'static str, serde_json::Value> = [ $($val),* ].into_iter().collect();
        $crate::metrics::Attributes::new(
            $target,
            $lvl,
            format_args!(""),
            file!(),
            line!(),
            column!(),
            module_path!(),
            attributes,
        )
    }};

    // === recursive case (more tts) ===

    // TODO(#1138): determine a new syntax for uninitialized span fields, and
    // re-enable this.
    // (@{ $(,)* $($out:expr),* }, $message:ident, $next:expr, $($k:ident).+ = _, $($rest:tt)*) => {
    //     $crate::valueset!($message:ident, @ { $($out),*, (&$next, None) }, $message:ident, $next, $($rest)*)
    // };
    // foo = ?bar ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($k:ident).+ = ?$val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), format!("{:?}", $val).into()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // foo = %bar ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($k:ident).+ = %$val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), format!("{}", $val).into()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // foo = bar ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($k:ident).+ = $val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), serde_json::to_value(&$val).unwrap()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // foo ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($k:ident).+, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), serde_json::to_value(&$($k).+).unwrap()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // ?foo ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, ?$($k:ident).+, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), format!("{:?}", $($k).+).into()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // %foo ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, %$($k:ident).+, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), format!("{}", $($k).+).into()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // foo = ?bar
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($k:ident).+ = ?$val:expr) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), format!("{:?}", $val).into()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };
    // foo = %bar
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($k:ident).+ = %$val:expr) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), format!("{}", $val).into()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };
    // foo = bar
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($k:ident).+ = $val:expr) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), serde_json::to_value(&$val).unwrap()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };
    // foo
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($k:ident).+) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), serde_json::to_value(&$($k).+).unwrap()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };
    // ?foo
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, ?$($k:ident).+) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), format!("{:?}", $($k).+).into()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };
    // %foo
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, %$($k:ident).+) => {
        $crate::valueset!(
            @ { $($out),*, (stringify!($($k).+), format!("{}", $($k).+).into()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };

    // Handle literal names
    // "foo" = ?bar ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $k:literal = ?$val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, ($k, format!("{:?}", $val).into()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // "foo" = %bar ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $k:literal = %$val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, ($k, format!("{}", $val).into()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // "foo" = bar ...
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $k:literal = $val:expr, $($rest:tt)*) => {
        $crate::valueset!(
            @ { $($out),*, ($k, serde_json::to_value(&$val).unwrap()) },
            $target,
            $lvl,
            $message,
            $next,
            $($rest)*
        )
    };
    // "foo" = ?bar
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $k:literal = ?$val:expr) => {
        $crate::valueset!(
            @ { $($out),*, ($k, format!("{:?}", $val).into()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };
    // "foo" = %bar
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $k:literal = %$val:expr) => {
        $crate::valueset!(
            @ { $($out),*, ($k, format!("{}", $val).into()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };
    // "foo" = bar
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $k:literal = $val:expr) => {
        $crate::valueset!(
            @ { $($out),*, ($k, serde_json::to_value(&$val).unwrap()) },
            $target,
            $lvl,
            $message,
            $next,
        )
    };

    // Remainder is unparseable, but exists --- must be format args!
    (@ { $(,)* $($out:expr),* }, $target:expr, $lvl:expr, $message:ident, $next:expr, $($rest:tt)+) => {{
        let attributes: std::collections::BTreeMap<&'static str, serde_json::Value> = [ $($out),* ].into_iter().collect();
        $crate::metrics::Attributes::new(
            $target,
            $lvl,
            format_args!($($rest)+),
            file!(),
            line!(),
            column!(),
            module_path!(),
            attributes,
        )
        // $message = Some(format_args!($($rest)+));
        // $crate::valueset!(
        //     @ { $($out),* },
        //     $message,
        //     $next,
        // )
    }};

    // === entry ===
    (target: $target:expr, $lvl:expr, $($kvs:tt)*) => {
        {
            // let mut message: Option<std::fmt::Arguments<'_>> = None;
            // let attributes: std::collections::BTreeMap<&'static str, serde_json::Value> = $crate::valueset!(
            //     @ { },
            //     message,
            //     (),
            //     $($kvs)+
            // );
            // (message, attributes)
            $crate::valueset!(
                @ { },
                $target,
                $lvl,
                message,
                (),
                $($kvs)*
            )
        }
    };
    // (entry: ) => {
    //     {
    //         (None, std::collections::BTreeMap::<&'static str, serde_json::Value>::new())
    //     }
    // };
    // (target: $target:expr, $lvl:expr, $($kvs:tt)*) => {{
    //     let mut message: Option<std::fmt::Arguments<'_>> = None;
    //     let attributes: std::collections::BTreeMap<&'static str, serde_json::Value> = $crate::valueset!(
    //         @ { },
    //         message,
    //         (),
    //         $($kvs)*
    //     );
    //     $crate::metrics::Attributes::new(
    //         $target,
    //         $lvl,
    //         message,
    //         file!(),
    //         line!(),
    //         column!(),
    //         module_path!(),
    //         attributes,
    //     )
    // }};
}

// #[doc(hidden)]
// #[macro_export]
// macro_rules! attributes {
//     (target: $target:expr, $lvl:expr, $($kvs:tt)*) => {{
//         let (message, attributes) = $crate::valueset!(entry: $($kvs)*);
//         $crate::metrics::Attributes::new(
//             $target,
//             $lvl,
//             message,
//             file!(),
//             line!(),
//             column!(),
//             module_path!(),
//             attributes,
//         )
//     }};
// }
