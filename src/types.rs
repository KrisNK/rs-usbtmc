//! ## Types
//!
//! The different types used across the crate
//!

use std::time::Duration;
use std::sync::{Arc, Mutex, MutexGuard};

use rusb::{Context, DeviceHandle, Direction, TransferType};

/// ### Handle
///
/// Alias for a libusb device handle wrapped in an Rc and RefCell.
/// 
#[derive(Debug, Clone)]
pub struct Handle(Arc<Mutex<DeviceHandle<Context>>>);

impl Handle {
    pub fn new(handle: DeviceHandle<Context>) -> Handle {
        Handle(Arc::new(Mutex::new(handle)))
    }

    pub fn borrow(&self) -> MutexGuard<'_, DeviceHandle<Context>> {
        self.0.lock().unwrap()
    }
}

/// ### Timeout
///
/// Alias for a duration wrapped in an Rc and RefCell.
#[derive(Debug, Clone)]
pub struct Timeout(Arc<Mutex<Duration>>);

impl Timeout {
    pub fn new(duration: Duration) -> Timeout {
        Timeout(Arc::new(Mutex::new(duration)))
    }

    pub fn borrow(&self) -> MutexGuard<'_, Duration> {
        self.0.lock().unwrap()
    }
}


/// ### bTag
///
/// The bTag element used to identify a bulk request.
///
/// Each time this value is called, it is incremented. If it increments past 255, it wraps around to 1.
///
#[derive(Debug, Clone)]
pub struct BTag(Arc<Mutex<u8>>);


impl BTag {
    /// ### New
    ///
    /// Return a fresh bTag set at the value 1.
    ///
    pub fn new() -> BTag {
        BTag(Arc::new(Mutex::new(1u8)))
    }

    /// ### Get
    ///
    /// Return the bTag value
    ///
    pub fn get(&self) -> u8 {
        let mut btag = self.0.lock().unwrap();
        let output = (*btag).clone();

        if *btag == 255 {
            *btag = 1;
        } else {
            *btag += 1;
        }

        output
    }
}

/// USB device address
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceAddr {
    /// USB bus number
    pub bus: u8,
    /// USB device number
    pub device: u8,
}

/// USB device identifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceId {
    /// USB Id Vendor
    pub vendor_id: u16,
    /// USB Id Product
    pub product_id: u16,
}

/// USB device info
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub address: DeviceAddr,
}

/// ### Device Mode
///
/// A collection of the configuration, interface and interface number. Also if the interface has a kernel driver attached.
///
#[derive(Debug, Clone, Default)]
pub struct DeviceMode {
    /// The USB configuration number
    pub config_number: u8,
    /// The interface number specific to the configuration
    pub interface_number: u8,
    /// The setting number specific to the interface
    pub setting_number: u8,
    /// If the device has a kernel driver. Important for returning control to the OS (on Linux).
    pub has_kernel_driver: bool,
}

/// ### Endpoint
///
/// Properties of an endpoint.
///
#[derive(Clone, Debug)]
pub struct Endpoint {
    /// Address of the endpoint on the interface
    pub address: u8,
    /// The maximal size a packet can have on this endpoint
    pub max_packet_size: u16,
    /// The transfer type of the endpoint (for USBTMC, Bulk or Interrupt)
    pub transfer_type: TransferType,
    /// The direction of the endpoint (for USBTMC, In or Out)
    pub direction: Direction,
}

/// ### USBTMC Endpoints
///
/// Endpoints specific to the USBTMC spec.
///
#[derive(Clone, Debug)]
pub struct UsbtmcEndpoints {
    /// The mandatory BULK OUT endpoint
    pub bulk_out_ep: Endpoint,
    /// The mandatory BULK IN endpoint
    pub bulk_in_ep: Endpoint,
    /// The optional INTERRUPT IN endpoint
    pub interrupt_ep: Option<Endpoint>,
}

/// ### Capabilities
///
/// The collected capabilities of a USBTMC device.
///
#[derive(Clone, Debug)]
pub struct Capabilities {
    pub bcd_version: u16,
    /// Can accept a control command for pulse
    pub accepts_indicator_pulse_request: bool,
    /// Only sends data to the controller
    pub is_talk_only: bool,
    /// Only accepts data from the controller
    pub is_listen_only: bool,
    /// When returning data, it has a terminator character in the data
    pub supports_bulk_in_term_char: bool,
}
