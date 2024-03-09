use bevy_ecs::prelude::*;

/// A trait similar to [`bevy_ecs::system::RunSystemOnce`], except it runs a system multiple times.
///
/// This is useful for testing multiple iterations of a system.
pub trait RunSystemLoop: Sized {
    /// Runs a system `n` times, returning a vector of the outputs.
    fn run_system_loop<T: IntoSystem<(), Out, Marker>, Out, Marker>(
        self,
        n: usize,
        system: T,
    ) -> Vec<Out> {
        self.run_system_loop_with(n, || (), system)
    }

    /// Runs a system `n` times with the given input, returning a vector of the outputs.
    fn run_system_loop_with<InputSource, T: IntoSystem<In, Out, Marker>, In, Out, Marker>(
        self,
        n: usize,
        input: InputSource,
        system: T,
    ) -> Vec<Out>
    where
        InputSource: Fn() -> In;
}

impl RunSystemLoop for &mut World {
    fn run_system_loop_with<InputSource, T: IntoSystem<In, Out, Marker>, In, Out, Marker>(
        self,
        n: usize,
        input: InputSource,
        system: T,
    ) -> Vec<Out>
    where
        InputSource: Fn() -> In,
    {
        let mut system: T::System = IntoSystem::into_system(system);
        system.initialize(self);
        let mut outs = Vec::new();
        for _ in 0..n {
            let out = system.run(input(), self);
            outs.push(out);
            system.apply_deferred(self);
        }
        outs
    }
}
