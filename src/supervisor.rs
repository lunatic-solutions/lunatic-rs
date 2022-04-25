use std::marker::PhantomData;

use crate::process::{AbstractProcess, ProcessRef, StartFailableProcess};
use crate::{host, Tag};

pub trait Supervisor
where
    Self: Sized,
{
    type Arg: serde::Serialize + serde::de::DeserializeOwned;
    type Children: Supervisable<Self>;

    fn init(config: &mut SupervisorConfig<Self>, arg: Self::Arg);
}

impl<T> AbstractProcess for T
where
    T: Supervisor,
{
    type Arg = T::Arg;
    type State = SupervisorConfig<T>;

    fn init(_: ProcessRef<Self>, arg: T::Arg) -> Self::State {
        // Supervisor shouldn't die if the children die
        unsafe { host::api::process::die_when_link_dies(0) };

        let mut config = SupervisorConfig::default();
        <T as Supervisor>::init(&mut config, arg);

        // Check if children arguments are configured inside of supervisor's `init` call.
        if config.children_args.is_none() {
            panic!(
                "SupervisorConfig<{0}>::children_args not set inside `{0}:init` function.",
                std::any::type_name::<T>()
            );
        }

        config
    }

    fn terminate(config: SupervisorConfig<T>) {
        config.terminate();
    }

    fn handle_link_trapped(config: &mut SupervisorConfig<T>, tag: Tag) {
        T::Children::handle_failure(config, tag);
    }
}

pub enum SupervisorStrategy {
    OneForOne,
}

pub struct SupervisorConfig<T>
where
    T: Supervisor,
{
    strategy: SupervisorStrategy,
    children: Option<<<T as Supervisor>::Children as Supervisable<T>>::Processes>,
    children_args: Option<<<T as Supervisor>::Children as Supervisable<T>>::Args>,
    children_tags: Option<<<T as Supervisor>::Children as Supervisable<T>>::Tags>,
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

    fn terminate(self) {
        T::Children::terminate(self);
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

impl<T1, K> Supervisable<K> for T1
where
    K: Supervisor<Children = Self>,
    T1: AbstractProcess,
    T1::Arg: Clone,
{
    type Processes = ProcessRef<T1>;
    type Args = (T1::Arg, Option<String>);
    type Tags = Tag;

    fn start_links(config: &mut SupervisorConfig<K>, args: Self::Args) {
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

    fn terminate(config: SupervisorConfig<K>) {
        config.children.unwrap().shutdown();
    }

    fn handle_failure(config: &mut SupervisorConfig<K>, tag: Tag) {
        match config.strategy {
            // After a failure, just restart the same process.
            SupervisorStrategy::OneForOne => {
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
    }
}

// Auto-implement Supervisable for up to 12 children.
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

    macro_rules! impl_supervisable {
        ($($args:ident $i:tt),*) => {
            impl<$($args),*, K> Supervisable<K> for ($($args),*)
            where
                K: Supervisor<Children = Self>,
                $(
                    $args : AbstractProcess,
                    $args ::Arg : Clone,
                )*
            {
                type Processes = ($(ProcessRef<$args>,)*);
                type Args = ($(($args ::Arg, Option<String>)),*);
                type Tags = ($(macros::tag!($args)),*);

                fn start_links(config: &mut SupervisorConfig<K>, args: Self::Args) {
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

                    config.children = Some(($(paste::paste!([<proc$i>])),*));
                    config.children_tags = Some(($(paste::paste!([<tag$i>])),*));
                }

                fn terminate(config: SupervisorConfig<K>) {
                    $( config.children.as_ref().unwrap().$i.shutdown() );*
                }

                fn handle_failure(config: &mut SupervisorConfig<K>, tag: Tag) {
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
                                    "Supervisor {} received kill signal",
                                    std::any::type_name::<K>()
                                );
                            }
                        }
                    }
                }
            }
        };
    }

    pub(crate) use impl_supervisable;
    pub(crate) use tag;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lunatic_test::test;

    use super::{Supervisor, SupervisorConfig};
    use crate::{
        process::{AbstractProcess, ProcessRef, StartProcess},
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

    impl Supervisor for SimpleSup {
        type Arg = ();
        type Children = SimpleServer;

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.children_args(((), None));
        }
    }

    #[test]
    fn supervisor_test() {
        SimpleSup::start_link((), None);
        sleep(Duration::from_millis(100));
    }
}
