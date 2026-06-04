/// Error returned by vtpm-wasm operations.
#[derive(Debug)]
pub enum VtpmError {
    /// A TPM-level error code from the WASM component.
    Tpm(u32),
    /// A WASM runtime or instantiation error.
    Runtime(wasmtime::Error),
}

impl std::fmt::Display for VtpmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tpm(code) => write!(f, "TPM error 0x{:08X}", code),
            Self::Runtime(e) => write!(f, "runtime error: {}", e),
        }
    }
}

impl std::error::Error for VtpmError {}

impl From<wasmtime::Error> for VtpmError {
    fn from(e: wasmtime::Error) -> Self {
        Self::Runtime(e)
    }
}
