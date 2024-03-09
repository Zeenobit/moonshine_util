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
        self.run_system_loop_with(n, (), system)
    }

    /// Runs a system `n` times with the given input, returning a vector of the outputs.
    fn run_system_loop_with<T: IntoSystem<In, Out, Marker>, In, Out, Marker>(
        self,
        n: usize,
        input: In,
        system: T,
    ) -> Vec<Out>
    where
        In: Clone;
}

impl RunSystemLoop for &mut World {
    fn run_system_loop_with<T: IntoSystem<In, Out, Marker>, In, Out, Marker>(
        self,
        n: usize,
        input: In,
        system: T,
    ) -> Vec<Out>
    where
        In: Clone,
    {
        let mut system: T::System = IntoSystem::into_system(system);
        system.initialize(self);
        let mut out = Vec::new();
        for _ in 0..n {
            out.push(system.run(input.clone(), self));
        }
        system.apply_deferred(self);
        out
    }
}
