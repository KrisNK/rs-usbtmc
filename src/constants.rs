//! ## Constants
//!
//! Various constants used throughout the project.
//!

#[allow(unused)]
pub mod usb {
    /// The class code for usbtmc
    pub const USBTMC_CLASS_CODE: u8 = 0xFE;
    /// The subclass code for usbtmc
    pub const USBTMC_SUBCLASS_CODE: u8 = 0x03;
    /// The protocol code for the USB488 spec of usbtmc
    pub const USBTMC_PROTOCOL_CODE: u8 = 0x01;
}

#[allow(unused)]
pub mod misc {
    use std::time::Duration;

    /// The default timeout duration
    pub const DEFAULT_TIMEOUT_DURATION: Duration = Duration::from_secs(2);
    /// The size in bytes of a USBTMC header in a bulk transfer
    pub const USBTMC_HEADER_SIZE: usize = 12;
    /// Buffer size we define for the application
    pub const APPLICATION_BUFFER_SIZE: u32 = 1024 * 8;
    /// Default termination character to use (using NI-VISA default '\n')
    pub const DEFAULT_TERM_CHAR: u8 = b'\n';
}

#[allow(unused)]
pub mod usbtmc_status {
    /// Success
    pub const STATUS_SUCCESS: u8 = 0x01;
    /// The device has received a split transaction CHECK_STATUS request and the request is being processed
    pub const STATUS_PENDING: u8 = 0x02;
    /// Failure for unspecified or undefined reason
    pub const STATUS_FAILED: u8 = 0x80;
    /// The device received an INITIATE_ABORT request, but the request is not in progress
    pub const STATUS_TRANSFER_NOT_IN_PROGRESS: u8 = 0x81;
    /// The device got a CHECK_STATUS request without any INITIATE request being processed
    pub const STATUS_SPLIT_NOT_IN_PROGRESS: u8 = 0x82;
    /// The device got an INIATE request, but another one is already being processed
    pub const STATUS_SPLIT_IN_PROGRESS: u8 = 0x83;
}

#[allow(unused)]
pub mod control_requests {
    pub const INITIATE_ABORT_BULK_OUT: u8 = 1;
    pub const CHECK_ABORT_BULK_OUT_STATUS: u8 = 2;
    pub const INITIATE_ABORT_BULK_IN: u8 = 3;
    pub const CHECK_ABORT_BULK_IN_STATUS: u8 = 4;
    pub const INITIATE_CLEAR: u8 = 5;
    pub const CHECK_CLEAR_STATUS: u8 = 6;
    pub const GET_CAPABILITIES: u8 = 7;
    pub const INDICATOR_PULSE: u8 = 64;
    pub const READ_STATUS_BYTE: u8 = 128;
}

#[allow(unused)]
pub mod bulk_msg_id {
    pub const DEVICE_DEPENDENT_MSG_OUT: u8 = 1;
    pub const REQUEST_DEVICE_DEPENDENT_MSG_IN: u8 = 2;
    pub const VENDOR_SPECIFIC_MSG_OUT: u8 = 126;
    pub const REQUEST_VENDOR_SPECIFIC_MSG_IN: u8 = 127;
    pub const DEVICE_DEPENDENT_MSG_IN: u8 = 2;
    pub const VENDOR_SPECIFIC_MSG_IN: u8 = 127;
}
