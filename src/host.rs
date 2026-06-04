/// Host-provided NVRAM storage for the `software-tpm` world.
///
/// The TPM component calls these methods during initialization and
/// operation to persist state across invocations.
pub trait StorageHandler: Send {
    /// Initialize the storage backend.
    fn init(&mut self) -> Result<(), u32>;

    /// Load a named state blob. Well-known names: `"permall"`,
    /// `"savestate"`, `"volatilestate"`.
    fn load_data(&mut self, tpm_number: u32, name: &str) -> Result<Vec<u8>, u32>;

    /// Store a named state blob.
    fn store_data(&mut self, data: &[u8], tpm_number: u32, name: &str) -> Result<(), u32>;

    /// Delete a named blob.
    fn delete_name(&mut self, tpm_number: u32, name: &str, must_exist: bool) -> Result<(), u32>;
}

/// Host-provided I/O context for the `software-tpm` world.
pub trait IoHandler: Send {
    /// Initialize the I/O subsystem.
    fn init(&mut self) -> Result<(), u32>;

    /// Get the current locality (0–4).
    fn get_locality(&mut self, tpm_number: u32) -> Result<u32, u32>;

    /// Get the physical presence assertion state.
    fn get_physical_presence(&mut self, tpm_number: u32) -> Result<bool, u32>;
}
