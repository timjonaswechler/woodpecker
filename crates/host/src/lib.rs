//! Private workspace package for the renderer-free controller and network host.
//! Definitions are shared with the public SDK; there is no second wire codec.
#[cfg(feature = "cli")]
#[path = "../../../src/cli/mod.rs"]
pub mod cli;
#[cfg(feature = "client")]
#[path = "../../../src/client/mod.rs"]
pub mod client;
#[path = "../../../src/command/mod.rs"]
pub mod command;
#[path = "../../../src/handle.rs"]
pub mod handle;
#[path = "../../../src/report/mod.rs"]
pub mod report;
#[cfg(feature = "server")]
#[path = "../../../src/server/mod.rs"]
pub mod server;
#[path = "../../../src/session/host_facade.rs"]
pub mod session;
/// Runtime capture shared by the Bevy adapter and the small native panic fixture.
#[doc(hidden)]
pub mod app_capture {
    pub use crate::report::capture::{error_layer, install_hook, layer_status};
}

#[cfg(test)]
#[path = "../../../src/test_support.rs"]
mod test_support;
