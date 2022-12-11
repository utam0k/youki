use libcontainer::workload::{default::DefaultExecutor, Executor};

pub fn default_executors() -> Vec<Box<dyn Executor>> {
    vec![
        #[cfg(feature = "wasm-wasmer")]
        Box::new(super::wasmer::WasmerExecutor::default()),
        #[cfg(feature = "wasm-wasmedge")]
        Box::new(super::wasmedge::WasmEdgeExecutor::default()),
        Box::new(DefaultExecutor::default()),
    ]
}
