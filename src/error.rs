//! ## USBTMC Errors
//!
//! The errors used throughout the crate.
//!

#[allow(unused)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("device not found")]
    DeviceNotFound,
    #[error("device is not compatible with USBTMC")]
    DeviceIncompatible,
    #[error("specified configuration not found")]
    ConfigurationNotFound,
    #[error("specified interface not found")]
    InterfaceNotFound,
    #[error("specified interface setting not found")]
    InterfaceSettingNotFound,
    #[error("bulk out endpoint not found")]
    BulkOutEndpointNotFound,
    #[error("bulk in endpoint not found")]
    BulkInEndpointNotFound,
    #[error("used incorrect endpoint")]
    IncorrectEndpoint,
    #[error("bulk in transfer cannot be aborted because FIFO is not empty")]
    BulkInFIFONotEmpty,
    #[error("no transfer in progress")]
    StatusNoTransferInProgress,
    #[error("control request failed")]
    StatusFailure,
    #[error("control request unexpectedly failed")]
    StatusUnexpectedFailure,
}
