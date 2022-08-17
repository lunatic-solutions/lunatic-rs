use std::marker::PhantomData;

use crate::process::Sendable;
use crate::{host, process::StartFields, protocol::ProtocolCapture, serializer::Bincode, Tag};
use crate::{
    process::{AbstractProcess, ProcessRef, StartFailableProcess, Subscriber},
    serializer::Serializer,
};

/// A `Supervisor` can detect failures (panics) inside [`AbstractProcesses`](AbstractProcess) and
/// restart them.
///
/// # Example
///
/// ```
/// struct Sup;
/// impl Supervisor for Sup {
///     type Arg = ();
///     // Start 3 `Counters` and monitor them for failures.
///     type Children = (Counter, Counter, Counter);
///
///     fn init(config: &mut SupervisorConfig<Self>, _: ()) {
///         // If a child fails, just restart it.
///         config.set_strategy(SupervisorStrategy::OneForOne);
///         // Start each `Counter` with a state of `0` & name last child "hello".
///         config.children_args((0, None),(0, None),(0, "hello".to_owned()));
///     }
/// }
///
/// let sup = Sup::start((), None);
/// let children = sup.children();
/// let count1 = children.2.request(Count);
/// // Get reference to named child.
/// let hello = ProcessRef::<Counter>::lookup("hello").unwrap();
/// let count2 = hello.request(Count);
/// assert_eq!(count1, count2);
/// ```
pub trait Supervisor<S = Bincode>
where
    Self: Sized,
{
    /// The argument received by the `init` function.
    ///
    /// This argument is sent from the parent to the child and needs to be serializable.
    type Arg: serde::Serialize + serde::de::DeserializeOwned;

    /// A tuple of types that implement `AbstractProcess`.
    ///
    /// They will be spawned as children. This can also include other supervisors.
    type Children: Supervisable<Self, S>;

    /// Entry function of the supervisor.
    ///
    /// It's used to configure the supervisor. The function `config.children_args()` must be called
    /// to provide arguments & names for children. If it's not called the supervisor will panic.
    fn init(config: &mut SupervisorConfig<Self, S>, arg: Self::Arg);
}

impl<T, S> AbstractProcess<S> for T
where
    T: Supervisor<S>,
    S: Serializer<()>,
{
    type Arg = T::Arg;
    type State = SupervisorConfig<T, S>;

    fn init(_: ProcessRef<Self, S>, arg: T::Arg) -> Self::State {
        // Supervisor shouldn't die if the children die
        unsafe { host::api::process::die_when_link_dies(0) };

        let mut config = SupervisorConfig::default();
        T::init(&mut config, arg);

        // Check if children arguments are configured inside of supervisor's `init` call.
        if config.children_args.is_none() {
            panic!(
                "SupervisorConfig<{0}>::children_args not set inside `{0}:init` function.",
                std::any::type_name::<T>()
            );
        }

        config
    }

    fn terminate(config: SupervisorConfig<T, S>) {
        config.terminate();
    }

    fn handle_link_trapped(config: &mut SupervisorConfig<T, S>, tag: Tag) {
        T::Children::handle_failure(config, tag);
    }
}

pub enum SupervisorStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

pub struct SupervisorConfig<T, S = Bincode>
where
    T: Supervisor<S>,
{
    strategy: SupervisorStrategy,
    children: Option<<T::Children as Supervisable<T, S>>::Processes>,
    children_args: Option<<T::Children as Supervisable<T, S>>::Args>,
    children_tags: Option<<T::Children as Supervisable<T, S>>::Tags>,
    terminate_subscribers: Vec<Subscriber<S>>,
    phantom: PhantomData<T>,
}

impl<T, S> SupervisorConfig<T, S>
where
    T: Supervisor<S>,
    S: Serializer<()>,
{
    pub fn set_strategy(&mut self, strategy: SupervisorStrategy) {
        self.strategy = strategy;
    }

    pub fn children_args(&mut self, args: <T::Children as Supervisable<T, S>>::Args) {
        T::Children::start_links(self, args)
    }

    pub(crate) fn get_children(&self) -> <T::Children as Supervisable<T, S>>::Processes {
        self.children.as_ref().unwrap().clone()
    }

    fn terminate(self) {
        self.terminate_subscribers
            .iter()
            .for_each(|sub| sub.notify());
        T::Children::terminate(self);
    }

    pub(crate) fn subscribe_shutdown(&mut self, subscriber: Subscriber<S>) {
        self.terminate_subscribers.push(subscriber);
    }
}

impl<T, S> Default for SupervisorConfig<T, S>
where
    T: Supervisor<S>,
{
    fn default() -> Self {
        SupervisorConfig {
            phantom: PhantomData,
            children: None,
            children_args: None,
            children_tags: None,
            terminate_subscribers: vec![],
            strategy: SupervisorStrategy::OneForOne,
        }
    }
}

pub trait Supervisable<T, S = Bincode>
where
    T: Supervisor<S>,
{
    type Processes: Clone;
    type Args: Clone;
    type Tags;

    fn start_links(config: &mut SupervisorConfig<T, S>, args: Self::Args);
    fn terminate(config: SupervisorConfig<T, S>);
    fn handle_failure(config: &mut SupervisorConfig<T, S>, tag: Tag);
}

macros::impl_supervisable_single!(crate::serializer::Bincode);
#[cfg(feature = "msgpack_serializer")]
macros::impl_supervisable_single!(crate::serializer::MessagePack);
#[cfg(feature = "json_serializer")]
macros::impl_supervisable_single!(crate::serializer::Json);

// Auto-implement Supervisable for up to 12 children.
macros::impl_supervisable!(T0 0);
macros::impl_supervisable!(T0 0, T1 1);
macros::impl_supervisable!(T0 0, T1 1, T2 2);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10, T11 11);
macros::impl_supervisable!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10, T11 11, T12 12);

mod macros {
    // Replace any identifier with `Tag`
    macro_rules! tag {
        ($args:ident) => {
            Tag
        };
    }

    macro_rules! reverse_shutdown {
        // reverse_shutdown!(config, [...]) shuts down all children in reverse order
        ($config:ident, []) => {}; // base case
        ($config:ident, [$head_i:tt $($rest_i:tt)*]) => { // recursive case
            macros::reverse_shutdown!($config, [$($rest_i)*]);
            $config.children.as_ref().unwrap().$head_i.shutdown();
        };
        // reverse_shutdown!(config, skip tag, [...]) shuts down all children with unmatched tags
        ($config:ident, skip $tag:ident, []) => {}; // base case
        ($config:ident, skip $tag:ident, [$head_i:tt $($rest_i:tt)*]) => { // recursive case
            macros::reverse_shutdown!($config, skip $tag, [$($rest_i)*]);
            if $tag != $config.children_tags.as_ref().unwrap().$head_i {
                $config.children.as_ref().unwrap().$head_i.shutdown();
            }
        };
        // reverse_shutdown!(config, after tag, [...]) shuts down the children after the tag
        ($config:ident, after $tag:ident, []) => {}; // base case
        ($config:ident, after $tag:ident, [$head_i:tt $($rest_i:tt)*]) => { // recursive case
            if $tag == $config.children_tags.as_ref().unwrap().$head_i {
                macros::reverse_shutdown!($config, [$($rest_i)*]);
            } else {
                macros::reverse_shutdown!($config, after $tag, [$($rest_i)*]);
            }
        };
    }

    macro_rules! impl_supervisable_single {
        ($serializer:path) => {
            impl<T1, K> Supervisable<K, $serializer> for T1
            where
                K: Supervisor<$serializer, Children = Self>,
                T1: AbstractProcess<$serializer>,
                T1::Arg: Clone,
            {
                type Processes = ProcessRef<T1, $serializer>;
                type Args = (T1::Arg, Option<String>);
                type Tags = Tag;

                fn start_links(config: &mut SupervisorConfig<K, $serializer>, args: Self::Args) {
                    config.children_args = Some(args.clone());
                    let (proc, tag) = match T1::start_link_or_fail(args.0, args.1.as_deref()) {
                        Ok(result) => result,
                        Err(_) => panic!(
                            "Supervisor failed to start child `{}`",
                            std::any::type_name::<T1>()
                        ),
                    };
                    config.children = Some(proc);
                    config.children_tags = Some(tag);
                }

                fn terminate(config: SupervisorConfig<K, $serializer>) {
                    config.children.unwrap().shutdown();
                }

                fn handle_failure(config: &mut SupervisorConfig<K, $serializer>, tag: Tag) {
                    // Since there is only one children process, the behavior is the same for all
                    // strategies -- after a failure, restart the child process
                    if tag == config.children_tags.unwrap() {
                        let (proc, tag) = match T1::start_link_or_fail(
                            config.children_args.as_ref().unwrap().0.clone(),
                            config.children_args.as_ref().unwrap().1.as_deref(),
                        ) {
                            Ok(result) => result,
                            Err(_) => panic!(
                                "Supervisor failed to start child `{}`",
                                std::any::type_name::<T1>()
                            ),
                        };
                        *config.children.as_mut().unwrap() = proc;
                        *config.children_tags.as_mut().unwrap() = tag;
                    } else {
                        panic!(
                            "Supervisor {} received kill signal",
                            std::any::type_name::<K>()
                        );
                    }
                }
            }
        };
    }

    macro_rules! impl_supervisable {
        ($($args:ident $i:tt),*) => {
            impl<$($args),*, K, S> Supervisable<K, S> for ($($args),*,)
            where
                K: Supervisor<S, Children = Self>,
                S: Serializer<()>
                    + Serializer<Sendable<S>>
                    + serde::Serialize
                    + serde::de::DeserializeOwned
                    $(
                        + Serializer<StartFields<$args, S>>
                        + Serializer<ProtocolCapture<StartFields<$args, S>, S>>
                    )*,
                $(
                    $args : AbstractProcess<S>,
                    $args ::Arg : Clone,
                )*
            {
                type Processes = ($(ProcessRef<$args, S>,)*);
                type Args = ($(($args ::Arg, Option<String>)),*,);
                type Tags = ($(macros::tag!($args)),*,);

                fn start_links(config: &mut SupervisorConfig<K, S>, args: Self::Args) {
                    config.children_args = Some(args.clone());

                    $(
                        let (paste::paste!([<proc$i>]),paste::paste!([<tag$i>]))
                                = match $args ::start_link_or_fail(args.$i.0, args.$i.1.as_deref()) {
                            Ok(result) => result,
                            Err(_) => panic!(
                                "Supervisor failed to start child `{}`",
                                std::any::type_name::<$args>()
                            ),
                        };
                    )*

                    config.children = Some(($(paste::paste!([<proc$i>])),*,));
                    config.children_tags = Some(($(paste::paste!([<tag$i>])),*,));
                }

                fn terminate(config: SupervisorConfig<K, S>) {
                    macros::reverse_shutdown!(config, [ $($i)* ]);
                }

                fn handle_failure(config: &mut SupervisorConfig<K, S>, tag: Tag) {
                    match config.strategy {
                        // After a failure, just restart the same process.
                        SupervisorStrategy::OneForOne => {

                            $(

                                if tag == config.children_tags.unwrap().$i {
                                    let (proc, tag) = match $args::start_link_or_fail(
                                        config.children_args.as_ref().unwrap().$i.0.clone(),
                                        config.children_args.as_ref().unwrap().$i.1.as_deref(),
                                    ) {
                                        Ok(result) => result,
                                        Err(_) => panic!(
                                            "Supervisor failed to start child `{}`",
                                            std::any::type_name::<$args>()
                                        ),
                                    };
                                    (*config.children.as_mut().unwrap()).$i = proc;
                                    (*config.children_tags.as_mut().unwrap()).$i = tag;
                                } else

                            )*

                            {
                                panic!(
                                    "Supervisor {} received kill signal from a died link",
                                    std::any::type_name::<K>()
                                );
                            }
                        }
                        // After a failure, restart all children
                        SupervisorStrategy::OneForAll => {
                            // check if the tag belongs to one of the children
                            $(
                                if tag == config.children_tags.unwrap().$i { } else
                            )*
                            {
                                panic!(
                                    "Supervisor {} received kill signal from a died link",
                                    std::any::type_name::<K>()
                                );
                            }

                            // shutdown children in reversed start order
                            macros::reverse_shutdown!(config, skip tag, [ $($i)* ]);

                            // restart all
                            $(

                                let (proc, tag) = match $args::start_link_or_fail(
                                    config.children_args.as_ref().unwrap().$i.0.clone(),
                                    config.children_args.as_ref().unwrap().$i.1.as_deref(),
                                ) {
                                    Ok(result) => result,
                                    Err(_) => panic!(
                                        "Supervisor failed to start child `{}`",
                                        std::any::type_name::<$args>()
                                    ),
                                };
                                (*config.children.as_mut().unwrap()).$i = proc;
                                (*config.children_tags.as_mut().unwrap()).$i = tag;

                            )*
                        }
                        // If a child process terminates, the rest of the child processes (that is,
                        // the child processes after the terminated process in start order)
                        // are terminated. Then the terminated child process and the rest of the
                        // child processes are restarted.
                        SupervisorStrategy::RestForOne => {
                            // check if the tag belongs to one of the children
                            $(
                                if tag == config.children_tags.unwrap().$i { } else
                            )*
                            {
                                panic!(
                                    "Supervisor {} received kill signal from a died link",
                                    std::any::type_name::<K>()
                                );
                            }

                            // shutdown children after the tag in reversed start order
                            macros::reverse_shutdown!(config, after tag, [ $($i)* ]);

                            // restart children starting at the tag
                            // silence false positive warnings on seen_tag
                            #[allow(unused_assignments)]
                            {
                                let mut seen_tag = false;
                                $(

                                    if seen_tag == true || tag == config.children_tags.unwrap().$i {
                                        seen_tag = true;

                                        let (proc, tag) = match $args::start_link_or_fail(
                                            config.children_args.as_ref().unwrap().$i.0.clone(),
                                            config.children_args.as_ref().unwrap().$i.1.as_deref(),
                                        ) {
                                            Ok(result) => result,
                                            Err(_) => panic!(
                                                "Supervisor failed to start child `{}`",
                                                std::any::type_name::<$args>()
                                            ),
                                        };
                                        (*config.children.as_mut().unwrap()).$i = proc;
                                        (*config.children_tags.as_mut().unwrap()).$i = tag;
                                    }

                                )*
                            }
                        }
                    }
                }
            }
        };
    }

    pub(crate) use impl_supervisable;
    pub(crate) use impl_supervisable_single;
    pub(crate) use reverse_shutdown;
    pub(crate) use tag;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lunatic_test::test;

    use super::{Supervisor, SupervisorConfig};
    use crate::{
        process::{AbstractProcess, ProcessRef, StartProcess},
        serializer::Bincode,
        sleep,
    };

    struct SimpleServer;

    impl AbstractProcess for SimpleServer {
        type Arg = ();
        type State = Self;

        fn init(_: ProcessRef<Self>, _arg: ()) -> Self::State {
            SimpleServer
        }
    }

    struct SimpleSup;

    impl Supervisor<Bincode> for SimpleSup {
        type Arg = ();
        type Children = SimpleServer;

        fn init(config: &mut SupervisorConfig<Self, Bincode>, _: ()) {
            config.children_args(((), None));
        }
    }

    #[test]
    fn supervisor_test() {
        SimpleSup::start_link((), None);
        sleep(Duration::from_millis(100));
    }
}
