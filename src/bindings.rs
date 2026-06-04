// Typed bindings for the ephemeral-tpm world (no host imports).
wasmtime::component::bindgen!({
    path: "../tpm-wit/wit",
    world: "ephemeral-tpm",
});
