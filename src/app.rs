use bevy_app::{App, Plugin, PluginGroupBuilder};

pub trait AddPluginFn {
    fn add_plugin_fn(self, f: impl FnPlugin) -> Self;
}

impl AddPluginFn for &mut App {
    fn add_plugin_fn(self, f: impl FnPlugin) -> Self {
        f(self);
        self
    }
}

pub trait AddPluginFnToGroup {
    fn add_fn(self, f: impl FnPlugin) -> Self;
}

impl AddPluginFnToGroup for PluginGroupBuilder {
    fn add_fn(self, f: impl FnPlugin) -> Self {
        self.add(FnPluginAdapter(f))
    }
}

pub trait FnPlugin: 'static + Fn(&mut App) + Send + Sync {}

impl<F> FnPlugin for F where F: 'static + Fn(&mut App) + Send + Sync {}

struct FnPluginAdapter<F>(F);

impl<F: FnPlugin> Plugin for FnPluginAdapter<F> {
    fn build(&self, app: &mut App) {
        self.0(app)
    }
}
