//! In-process virtual TPM 2.0 host via libtpms WASM component.
//!
//! This crate loads a WASM component that implements the
//! `tegmentum:tpm/ephemeral-tpm` world and exposes every WIT-defined
//! function as a typed Rust method.
//!
//! No TPM command building or protocol knowledge lives here — this is
//! purely a host wrapper for the WIT contract.

mod bindings;
mod engine;
mod error;
mod host;

pub use engine::VtpmEngine;
pub use error::VtpmError;
pub use host::{IoHandler, StorageHandler};

// Re-export WIT-generated types that appear in the public API.
pub use bindings::tegmentum::tpm::types::{
    BlobType, BufferSizeInfo, InfoFlags, StateType, StateValidationFlags, TpmError, TpmProperty,
    TpmVersion,
};
