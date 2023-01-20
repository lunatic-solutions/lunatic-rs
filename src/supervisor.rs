use std::marker::PhantomData;

use crate::ap::handlers::{DeferredRequest, Request};
use crate::ap::{
    AbstractProcess, Config, DeferredRequestHandler, DeferredResponse, ProcessRef, RequestHandler,
    State,
};
use crate::serializer::Bincode;
use crate::{host, Tag};

/// A `Supervisor` can detect failures (panics) inside
/// [`AbstractProcesses`](AbstractProcess) and restart them.
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
pub trait Supervisor
where
    Self: Sized,
{
    /// The argument received by the `init` function.
    ///
    /// This argument is sent from the parent to the child and needs to be
    /// serializable.
    type Arg: serde::Serialize + serde::de::DeserializeOwned;

    /// A tuple of types that implement `AbstractProcess`.
    ///
    /// They will be spawned as children. This can also include other
    /// supervisors.
    type Children: Supervisable<Self>;

    /// Entry function of the supervisor.
    ///
    /// It's used to configure the supervisor. The function
    /// `config.children_args()` must be called to provide arguments & names
    /// for children. If it's not called the supervisor will panic.
    fn init(config: &mut SupervisorConfig<Self>, arg: Self::Arg);
}

impl<T> AbstractProcess for T
where
    T: Supervisor,
{
    type Arg = T::Arg;
    type State = SupervisorConfig<T>;
    type Serializer = Bincode;
    type Handlers = (Request<GetChildren>, DeferredRequest<ShutdownSubscribe>);
    type StartupError = ();

    fn init(config: Config<Self>, arg: T::Arg) -> Result<Self::State, ()> {
        // Supervisor shouldn't die if the children die
        config.die_if_link_dies(false);

        let mut sup_config = SupervisorConfig::default();
        <T as Supervisor>::init(&mut sup_config, arg);

        // Check if children arguments are configured inside of supervisor's `init`
        // call.
        if sup_config.children_args.is_none() {
            panic!(
                "SupervisorConfig<{0}>::children_args not set inside `{0}:init` function.",
                std::any::type_name::<T>()
            );
        }

        Ok(sup_config)
    }

    fn terminate(config: SupervisorConfig<T>) {
        config.terminate();
    }

    fn handle_link_death(sup_config: &mut SupervisorConfig<T>, tag: Tag) {
        T::Children::handle_failure(sup_config, tag);
    }
}

impl<T> ProcessRef<T>
where
    T: Supervisor,
    T: AbstractProcess<State = SupervisorConfig<T>, Serializer = Bincode>,
{
    /// Blocks until the Supervisor shuts down.
    ///
    /// A tagged message will be sent to the supervisor process as a request
    /// and the subscription will be registered. When the supervisor process
    /// shuts down, the subscribers will be each notified by a response
    /// message and therefore be unblocked after having received the awaited
    /// message.
    pub fn wait_on_shutdown(&self) {
        self.deferred_request(ShutdownSubscribe, None).unwrap();
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ShutdownSubscribe;
impl<T> DeferredRequestHandler<ShutdownSubscribe> for T
where
    T: Supervisor,
    T: AbstractProcess<State = SupervisorConfig<T>, Serializer = Bincode>,
{
    type Response = ();

    fn handle(
        mut state: State<Self>,
        _: ShutdownSubscribe,
        subscriber: DeferredResponse<(), Self::Serializer>,
    ) {
        state.subscribe_shutdown(subscriber)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GetChildren;
impl<T> RequestHandler<GetChildren> for T
where
    T: Supervisor,
    T: AbstractProcess<State = SupervisorConfig<T>, Serializer = Bincode>,
{
    type Response = <<T as Supervisor>::Children as Supervisable<T>>::Processes;

    fn handle(state: State<Self>, _: GetChildren) -> Self::Response {
        state.get_children()
    }
}

impl<T> ProcessRef<T>
where
    T: Supervisor,
    T: AbstractProcess<State = SupervisorConfig<T>, Serializer = Bincode>,
{
    pub fn children(&self) -> <<T as Supervisor>::Children as Supervisable<T>>::Processes {
        self.request(GetChildren, None).unwrap()
    }
}

pub enum SupervisorStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

pub struct SupervisorConfig<T>
where
    T: Supervisor,
{
    strategy: SupervisorStrategy,
    children: Option<<<T as Supervisor>::Children as Supervisable<T>>::Processes>,
    children_args: Option<<<T as Supervisor>::Children as Supervisable<T>>::Args>,
    children_tags: Option<<<T as Supervisor>::Children as Supervisable<T>>::Tags>,
    terminate_subscribers: Vec<DeferredResponse<(), Bincode>>,
    phantom: PhantomData<T>,
}

impl<T> SupervisorConfig<T>
where
    T: Supervisor,
{
    pub fn set_strategy(&mut self, strategy: SupervisorStrategy) {
        self.strategy = strategy;
    }

    pub fn children_args(&mut self, args: <<T as Supervisor>::Children as Supervisable<T>>::Args) {
        T::Children::start_links(self, args)
    }

    pub(crate) fn get_children(
        &self,
    ) -> <<T as Supervisor>::Children as Supervisable<T>>::Processes {
        self.children.as_ref().unwrap().clone()
    }

    fn terminate(mut self) {
        self.terminate_subscribers
            .drain(..)
            .for_each(|sub| sub.send_response(()));
        T::Children::terminate(self);
    }

    pub(crate) fn subscribe_shutdown(&mut self, subscriber: DeferredResponse<(), Bincode>) {
        self.terminate_subscribers.push(subscriber);
    }
}

impl<T> Default for SupervisorConfig<T>
where
    T: Supervisor,
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

pub trait Supervisable<T>
where
    T: Supervisor,
{
    type Processes: serde::Serialize + serde::de::DeserializeOwned + Clone;
    type Args: Clone;
    type Tags;

    fn start_links(config: &mut SupervisorConfig<T>, args: Self::Args);
    fn terminate(config: SupervisorConfig<T>);
    fn handle_failure(config: &mut SupervisorConfig<T>, tag: Tag);
}

// Implement Supervisable for tuples with up to 12 children.
macros::impl_supervisable!();
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
        ($t:ident) => {
            Tag
        };
    }

    macro_rules! reverse_shutdown {
        // reverse_shutdown!(config, [...]) shuts down all children in reverse order
        ($config:ident, []) => {}; // base case
        ($config:ident, [$head_i:tt $($rest_i:tt)*]) => { // recursive case
            macros::reverse_shutdown!($config, [$($rest_i)*]);
            $config.children.as_ref().unwrap().$head_i.shutdown(None).unwrap();
        };
        // reverse_shutdown!(config, skip tag, [...]) shuts down all children with unmatched tags
        ($config:ident, skip $tag:ident, []) => {}; // base case
        ($config:ident, skip $tag:ident, [$head_i:tt $($rest_i:tt)*]) => { // recursive case
            macros::reverse_shutdown!($config, skip $tag, [$($rest_i)*]);
            if $tag != $config.children_tags.as_ref().unwrap().$head_i {
                $config.children.as_ref().unwrap().$head_i.shutdown(None).unwrap();
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

    macro_rules! impl_supervisable {
        ($($t:ident $i:tt),*) => {
            impl<$($t,)* K> Supervisable<K> for ($($t,)*)
            where
                K: Supervisor<Children = Self>,
                $(
                    $t : AbstractProcess,
                    $t ::Arg : Clone,
                )*
            {
                type Processes = ($(ProcessRef<$t>,)*);
                type Args = ($(($t ::Arg, Option<String>),)*);
                type Tags = ($(macros::tag!($t),)*);

                fn start_links(config: &mut SupervisorConfig<K>, args: Self::Args) {
                    config.children_args = Some(args.clone());

                    $(
                        let paste::paste!([<tag$i>]) = Tag::new();
                        let result = match args.$i.1 {
                            Some(name) => $t::link_with(paste::paste!([<tag$i>])).start_as(name, args.$i.0),
                            None => $t::link_with(paste::paste!([<tag$i>])).start(args.$i.0),
                        };
                        let paste::paste!([<proc$i>]) = match result {
                            Ok(proc) => proc,
                            Err(err) => panic!("Supervisor failed to start child `{:?}`", err),
                        };
                    )*
                    config.children = Some(($(paste::paste!([<proc$i>]),)*));
                    config.children_tags = Some(($(paste::paste!([<tag$i>]),)*));
                }

                #[allow(unused_variables)]
                fn terminate(config: SupervisorConfig<K>) {
                    macros::reverse_shutdown!(config, [ $($i)* ]);
                }

                #[allow(unused_variables)]
                fn handle_failure(config: &mut SupervisorConfig<K>, tag: Tag) {
                    match config.strategy {
                        // After a failure, just restart the same process.
                        SupervisorStrategy::OneForOne => {

                            $(

                                if tag == config.children_tags.unwrap().$i {
                                    let args = (
                                        config.children_args.as_ref().unwrap().$i.0.clone(),
                                        config.children_args.as_ref().unwrap().$i.1.as_deref(),
                                    );
                                    let link_tag = Tag::new();
                                    let result = match args.1 {
                                        Some(name) => {
                                            // Remove first the previous registration
                                            let remove = format!("{} + ProcessRef + {}", name, std::any::type_name::<$t>());
                                            unsafe { host::api::registry::remove(remove.as_ptr(), remove.len()) };
                                            $t::link().start_as(name, args.0)
                                        },
                                        None => $t::link_with(link_tag).start(args.0),
                                    };
                                    let proc = match result {
                                        Ok(proc) => proc,
                                        Err(err) => panic!("Supervisor failed to start child `{:?}`", err),
                                    };
                                    config.children.as_mut().unwrap().$i = proc;
                                    config.children_tags.as_mut().unwrap().$i = link_tag;
                                } else

                            )*

                            {
                                panic!(
                                    "Supervisor {} received link death signal not belonging to a child",
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
                                    "Supervisor {} received link death signal not belonging to a child",
                                    std::any::type_name::<K>()
                                );
                            }

                            // shutdown children in reversed start order
                            macros::reverse_shutdown!(config, skip tag, [ $($i)* ]);

                            // restart all
                            $(

                                let args = (
                                    config.children_args.as_ref().unwrap().$i.0.clone(),
                                    config.children_args.as_ref().unwrap().$i.1.as_deref(),
                                );
                                let link_tag = Tag::new();
                                let result = match args.1 {
                                    Some(name) => {
                                        // Remove first the previous registration
                                        let remove = format!("{} + ProcessRef + {}", name, std::any::type_name::<$t>());
                                        unsafe { host::api::registry::remove(remove.as_ptr(), remove.len()) };
                                        $t::link().start_as(name, args.0)
                                    },
                                    None => $t::link_with(link_tag).start(args.0),
                                };
                                let proc = match result {
                                    Ok(proc) => proc,
                                    Err(err) => panic!("Supervisor failed to start child `{:?}`", err),
                                };
                                config.children.as_mut().unwrap().$i = proc;
                                config.children_tags.as_mut().unwrap().$i = link_tag;

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
                                    "Supervisor {} received link death signal not belonging to a child",
                                    std::any::type_name::<K>()
                                );
                            }

                            // shutdown children after the tag in reversed start order
                            macros::reverse_shutdown!(config, after tag, [ $($i)* ]);

                            // restart children starting at the tag
                            #[allow(unused_assignments, unused_variables, unreachable_code)]
                            {
                                let mut seen_tag = false;
                                $(

                                    if seen_tag == true || tag == config.children_tags.unwrap().$i {
                                        seen_tag = true;

                                        let args = (
                                            config.children_args.as_ref().unwrap().$i.0.clone(),
                                            config.children_args.as_ref().unwrap().$i.1.as_deref(),
                                        );
                                        let link_tag = Tag::new();
                                        let result = match args.1 {
                                            Some(name) => {
                                                // Remove first the previous registration
                                                let remove = format!("{} + ProcessRef + {}", name, std::any::type_name::<$t>());
                                                unsafe { host::api::registry::remove(remove.as_ptr(), remove.len()) };
                                                $t::link().start_as(name, args.0)
                                            },
                                            None => $t::link_with(link_tag).start(args.0),
                                        };
                                        let proc = match result {
                                            Ok(proc) => proc,
                                            Err(err) => panic!("Supervisor failed to start child `{:?}`", err),
                                        };
                                        config.children.as_mut().unwrap().$i = proc;
                                        config.children_tags.as_mut().unwrap().$i = link_tag;

                                    }

                                )*
                            }
                        }
                    }
                }
            }
        };
    }

    pub(crate) use {impl_supervisable, reverse_shutdown, tag};
}

#[cfg(test)]
mod tests {
    use lunatic_test::test;

    use super::{Supervisor, SupervisorConfig};
    use crate::ap::{AbstractProcess, Config};
    use crate::serializer::Bincode;

    struct SimpleServer;

    impl AbstractProcess for SimpleServer {
        type Arg = ();
        type State = Self;
        type Serializer = Bincode;
        type Handlers = ();
        type StartupError = ();

        fn init(_: Config<Self>, _arg: ()) -> Result<Self, ()> {
            Ok(SimpleServer)
        }
    }

    struct SimpleSup;

    impl Supervisor for SimpleSup {
        type Arg = ();
        type Children = (SimpleServer,);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.children_args((((), None),));
        }
    }

    #[test]
    fn supervisor_test() {
        SimpleSup::link().start(()).unwrap();
    }
}
