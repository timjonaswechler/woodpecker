//! Compile-time proof of the panic strategy; no process::abort() fixture mode.
#[cfg(not(panic = "abort"))]
compile_error!("panic_abort_fixture must be built with panic=abort");

#[path = "../../../failure_source.rs"]
mod observation;

fn main() {
    observation::main();
}
