use std::marker::PhantomData;

use crate::server::StartFailableServer;
use crate::{host, Server, Tag};

pub trait Supervisor
where
    Self: Sized,
{
    type Arg: serde::Serialize + serde::de::DeserializeOwned;
    type Children: Supervisable;

    fn init(config: &mut SupervisorConfig<Self>, arg: Self::Arg);
}

impl<T> Server for T
where
    T: Supervisor,
{
    type Arg = T::Arg;
    type State = SupervisorConfig<T>;

    fn init(arg: T::Arg) -> Self::State {
        // Supervisor shouldn't die if the children die
        unsafe { host::api::process::die_when_link_dies(0) };

        let mut config = SupervisorConfig::default();
        <T as Supervisor>::init(&mut config, arg);

        config
    }

    fn handle_link_trapped(_state: &mut SupervisorConfig<T>, _tag: Tag) {}
}

pub enum SupervisorStrategy {
    OneForOne,
}

pub struct SupervisorConfig<T>
where
    T: Supervisor,
{
    strategy: SupervisorStrategy,
    phantom: PhantomData<T>,
}

impl<T> SupervisorConfig<T>
where
    T: Supervisor,
{
    pub fn set_strategy(&mut self, strategy: SupervisorStrategy) {
        self.strategy = strategy;
    }

    pub fn set_children(&mut self, children: <<T as Supervisor>::Children as Supervisable>::Args) {
        T::Children::start_links(self, children)
    }
}

impl<T> Default for SupervisorConfig<T>
where
    T: Supervisor,
{
    fn default() -> Self {
        SupervisorConfig {
            phantom: PhantomData,
            strategy: SupervisorStrategy::OneForOne,
        }
    }
}

pub trait Supervisable: Sized {
    type Args: Clone;

    fn start_links<T: Supervisor>(config: &mut SupervisorConfig<T>, args: Self::Args);
    fn get_children(&self);
}

impl<T1> Supervisable for T1
where
    T1: Server,
    T1::Arg: Clone,
{
    type Args = T1::Arg;

    fn start_links<T: Supervisor>(config: &mut SupervisorConfig<T>, args: Self::Args) {
        if let Err(_) = T1::start_link_or_fail(args.clone(), None) {
            panic!(
                "Supervisor failed to start child `{}`",
                std::any::type_name::<T1>()
            );
        }
    }

    fn get_children(&self) {}
}

impl<T1, T2> Supervisable for (T1, T2)
where
    T1: Server,
    T1::Arg: Clone,
    T2: Server,
    T2::Arg: Clone,
{
    type Args = (T1::Arg, T2::Arg);

    fn start_links<T: Supervisor>(config: &mut SupervisorConfig<T>, args: Self::Args) {
        if let Err(_) = T1::start_link_or_fail(args.0.clone(), None) {
            panic!(
                "Supervisor failed to start child `{}`",
                std::any::type_name::<T1>()
            );
        }
        if let Err(_) = T2::start_link_or_fail(args.1.clone(), None) {
            panic!(
                "Supervisor failed to start child `{}`",
                std::any::type_name::<T2>()
            );
        };
    }

    fn get_children(&self) {}
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lunatic_test::test;

    use super::{Supervisor, SupervisorConfig};
    use crate::{sleep, Server, StartServer};

    struct SimpleServer;

    impl Server for SimpleServer {
        type Arg = ();
        type State = Self;

        fn init(_arg: ()) -> Self::State {
            SimpleServer
        }
    }

    struct SimpleSup;

    impl Supervisor for SimpleSup {
        type Arg = ();
        type Children = SimpleServer;

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_children(());
        }
    }

    #[test]
    fn supervisor_test() {
        SimpleSup::start_link((), None);
        sleep(Duration::from_millis(100));
    }
}
