use std::path::Path;

use wasmtime::component::Component;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::bindings::tegmentum::tpm::types::{
    BlobType, BufferSizeInfo, InfoFlags, StateType, StateValidationFlags, TpmProperty, TpmVersion,
};
use crate::bindings::EphemeralTpm;
use crate::error::VtpmError;

struct WasmState {
    wasi: WasiCtx,
    table: wasmtime::component::ResourceTable,
}

impl WasiView for WasmState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// In-process virtual TPM engine backed by a WASM component that
/// implements the `tegmentum:tpm/ephemeral-tpm` world.
///
/// All methods take `&mut self` — callers are responsible for
/// synchronization if shared across threads.
pub struct VtpmEngine {
    store: Store<WasmState>,
    instance: EphemeralTpm,
}

impl VtpmEngine {
    /// Load an ephemeral-tpm WASM component from disk.
    pub fn new(component_path: &Path) -> Result<Self, VtpmError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).map_err(VtpmError::Runtime)?;

        let wasi = WasiCtxBuilder::new().inherit_stderr().build();
        let state = WasmState {
            wasi,
            table: wasmtime::component::ResourceTable::new(),
        };
        let mut store = Store::new(&engine, state);

        let component = Component::from_file(&engine, component_path)
            .map_err(VtpmError::Runtime)?;
        let mut linker = wasmtime::component::Linker::<WasmState>::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
            .map_err(VtpmError::Runtime)?;

        let instance = EphemeralTpm::instantiate(&mut store, &component, &linker)
            .map_err(VtpmError::Runtime)?;

        Ok(Self { store, instance })
    }

    /// Load an ephemeral-tpm WASM component from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, VtpmError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).map_err(VtpmError::Runtime)?;

        let wasi = WasiCtxBuilder::new().inherit_stderr().build();
        let state = WasmState {
            wasi,
            table: wasmtime::component::ResourceTable::new(),
        };
        let mut store = Store::new(&engine, state);

        let component = Component::from_binary(&engine, bytes)
            .map_err(VtpmError::Runtime)?;
        let mut linker = wasmtime::component::Linker::<WasmState>::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
            .map_err(VtpmError::Runtime)?;

        let instance = EphemeralTpm::instantiate(&mut store, &component, &linker)
            .map_err(VtpmError::Runtime)?;

        Ok(Self { store, instance })
    }

    // -- lifecycle --

    /// Get the packed version of the TPM implementation.
    pub fn get_version(&mut self) -> Result<u32, VtpmError> {
        self.instance
            .tegmentum_tpm_lifecycle()
            .call_get_version(&mut self.store)
            .map_err(Into::into)
    }

    /// Select the TPM specification version before init.
    pub fn choose_version(&mut self, version: TpmVersion) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_lifecycle()
            .call_choose_version(&mut self.store, version)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Initialize the TPM after choose-version and state setup.
    pub fn init_tpm(&mut self) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_lifecycle()
            .call_init(&mut self.store)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Shutdown and release TPM resources.
    pub fn terminate(&mut self) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_lifecycle()
            .call_terminate(&mut self.store)
            .map_err(Into::into)
    }

    /// Returns true if the TPM has valid permanent state.
    pub fn was_manufactured(&mut self) -> Result<bool, VtpmError> {
        self.instance
            .tegmentum_tpm_lifecycle()
            .call_was_manufactured(&mut self.store)
            .map_err(Into::into)
    }

    /// Decode a base64-encoded state blob to raw binary.
    pub fn decode_blob(
        &mut self,
        data: &str,
        blob_type: BlobType,
    ) -> Result<Vec<u8>, VtpmError> {
        self.instance
            .tegmentum_tpm_lifecycle()
            .call_decode_blob(&mut self.store, data, blob_type)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    // -- commands --

    /// Submit a raw TPM command and receive the response.
    pub fn process(&mut self, command: &[u8]) -> Result<Vec<u8>, VtpmError> {
        self.instance
            .tegmentum_tpm_commands()
            .call_process(&mut self.store, command)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Request cancellation of the currently executing command.
    pub fn cancel_command(&mut self) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_commands()
            .call_cancel_command(&mut self.store)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    // -- state --

    /// Retrieve a TPM state blob.
    pub fn get_state(&mut self, st: StateType) -> Result<Vec<u8>, VtpmError> {
        self.instance
            .tegmentum_tpm_state()
            .call_get_state(&mut self.store, st)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Load a state blob before init.
    pub fn set_state(&mut self, st: StateType, data: &[u8]) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_state()
            .call_set_state(&mut self.store, st, data)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Validate state blob compatibility before init.
    pub fn validate_state(
        &mut self,
        flags: StateValidationFlags,
        extra: u32,
    ) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_state()
            .call_validate_state(&mut self.store, flags, extra)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Store all volatile state in a single blob for migration.
    pub fn volatile_all_store(&mut self) -> Result<Vec<u8>, VtpmError> {
        self.instance
            .tegmentum_tpm_state()
            .call_volatile_all_store(&mut self.store)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    // -- config --

    /// Query a TPM property value.
    pub fn get_property(&mut self, prop: TpmProperty) -> Result<i32, VtpmError> {
        self.instance
            .tegmentum_tpm_config()
            .call_get_property(&mut self.store, prop)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Set the command/response buffer size.
    pub fn set_buffer_size(&mut self, wanted: u32) -> Result<BufferSizeInfo, VtpmError> {
        self.instance
            .tegmentum_tpm_config()
            .call_set_buffer_size(&mut self.store, wanted)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Set the operational profile (JSON string).
    pub fn set_profile(&mut self, profile: &str) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_config()
            .call_set_profile(&mut self.store, profile)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Query TPM info as JSON.
    pub fn get_info(&mut self, flags: InfoFlags) -> Result<String, VtpmError> {
        self.instance
            .tegmentum_tpm_config()
            .call_get_info(&mut self.store, flags)
            .map_err(Into::into)
    }

    /// Set debug verbosity (0 disables).
    pub fn set_debug_level(&mut self, level: u32) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_config()
            .call_set_debug_level(&mut self.store, level)
            .map_err(Into::into)
    }

    /// Set a prefix for debug messages.
    pub fn set_debug_prefix(&mut self, prefix: &str) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_config()
            .call_set_debug_prefix(&mut self.store, prefix)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    // -- tis --

    /// Begin hash sequence for extend-to-PCR-17 at locality 4.
    pub fn hash_start(&mut self) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_tis()
            .call_hash_start(&mut self.store)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Feed data into the active hash.
    pub fn hash_data(&mut self, data: &[u8]) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_tis()
            .call_hash_data(&mut self.store, data)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Complete hash and extend PCR 17.
    pub fn hash_end(&mut self) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_tis()
            .call_hash_end(&mut self.store)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Query TPM Established bit.
    pub fn tpm_established_get(&mut self) -> Result<bool, VtpmError> {
        self.instance
            .tegmentum_tpm_tis()
            .call_tpm_established_get(&mut self.store)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }

    /// Reset TPM Established bit (only from locality 3–4).
    pub fn tpm_established_reset(&mut self) -> Result<(), VtpmError> {
        self.instance
            .tegmentum_tpm_tis()
            .call_tpm_established_reset(&mut self.store)
            .map_err(VtpmError::Runtime)?
            .map_err(VtpmError::Tpm)
    }
}
