//! # Rust USBTMC
//!
//! Pure Rust implementation of the USBTMC protocol to connect to instruments.
//!
//! Thus far, this library implements the basic USBTMC control endpoint commands,
//! writing DEVICE_DEPENDENT messages to the BULK OUT endpoint and reading DEVICE_DEPENDENT
//! messages to the BULK IN endpoint.
//!
//! ## Usage
//!
//! To use, add the following line to your project's Cargo.toml dependencies:
//! ```toml
//! rs-usbtmc = "0.1"
//! ```
//!
//! ## Example
//!
//! The example below demonstrates how to connect to, send commands to and query the device.
//!
//! ```rust
//! use rs_usbtmc::UsbtmcClient;
//!
//! const DEVICE_VID: u16 = 0x0000;
//! const DEVICE_PID: u16 = 0x0000;
//!
//! fn main() {
//!     // connect to the device
//!     let device = UsbtmcClient::connect(DEVICE_VID, DEVICE_PID).expect("failed to connect");
//!
//!     // send a command to the device
//!     device.command("*IDN?").expect("failed to send command");
//!
//!     // query the device and get a string
//!     let response: String = device.query("*IDN?").expect("failed to query device");
//!
//!     // query the device and get a bytes
//!     let response: Vec<u8> = device.query_raw("*IDN?").expect("failed to query device");
//! }
//! ```
//!
//! ## Project Plans
//!
//! I created this driver as part of a project to control an oscilloscope during a summer
//! research position. Alone, I do not have access to //! an oscilloscope.
//! If I do obtain one, the plan is to:
//!
//! - Fully implement all possible requests
//! - Implement the usb488 subclass requests
//!
//! I'll reach out to my university for access to an instrument to complete this project, but I'm open to collaborating.
//!

mod constants;
mod error;
mod init;
mod types;
mod communication {
    pub mod bulk;
    pub mod control;
}

use rusb::DeviceDescriptor;
pub use types::{DeviceAddr, DeviceId, DeviceInfo};

use communication::control;
use constants::misc::DEFAULT_TIMEOUT_DURATION;
use types::{BTag, Capabilities, DeviceMode, Handle, Timeout, UsbtmcEndpoints};

use anyhow::Result;

/// Device filter
pub trait DeviceFilter {
    fn apply_filter<T: rusb::UsbContext>(
        &self,
        device: &rusb::Device<T>,
        device_desc: &DeviceDescriptor,
    ) -> bool;
}

/// ### UsbtmcClient
///
/// Client connected to a USBTMC device.
///
#[derive(Debug)]
pub struct UsbtmcClient {
    handle: Handle,
    mode: DeviceMode,
    timeout: Timeout,
    capabilities: Capabilities,
    btag: BTag,
    endpoints: UsbtmcEndpoints,
}

impl UsbtmcClient {
    /// ### TMC devices
    ///
    /// Get a list of USB TMC devices
    ///
    pub fn devices() -> Result<Vec<DeviceInfo>> {
        // setup context
        let mut context = rusb::Context::new()?;

        init::list_devices(&mut context)
    }

    /// ### Connect
    ///
    /// Connect a USB device and initialize it.
    ///
    /// Use `filter` argument to select instrument device:
    /// - `()` - first found USBTMC device
    /// - `(idVendor, idProduct)` or `DeviceId` - device by USB identifiers
    /// - `(bus, device)` or `DeviceAddr` - device by USB bus and device number
    /// - `DeviceInfo` - device by both USB identifiers and address
    ///
    pub fn connect(filter: impl DeviceFilter) -> Result<UsbtmcClient> {
        // setup context
        let mut context = rusb::Context::new()?;
        // attempt to open the device
        let (device, mut handle) = init::open_device(&mut context, filter)?;

        // GET THE DEVICE MODE
        // ==========

        // get the mode
        let mut mode = init::get_usbtmc_mode(&device)?;
        // detach kernel driver if it is used
        init::detach_kernel_driver(&mut mode, &mut handle)?;

        // GET ENDPOINTS
        // ==========
        let endpoints: UsbtmcEndpoints = init::get_endpoints(&mode, &device)?;

        // CONFIGURE DEVICE
        // ==========
        handle.set_active_configuration(mode.config_number)?;
        handle.claim_interface(mode.interface_number)?;
        handle.set_alternate_setting(mode.interface_number, mode.setting_number)?;

        // SETUP DATA FOR CLIENT
        // ==========
        let handle: Handle = Handle::new(handle);
        let timeout: Timeout = Timeout::new(DEFAULT_TIMEOUT_DURATION);
        let btag = BTag::new();

        // GET CAPABILITIES
        // ==========
        let capabilities: Capabilities =
            control::get_capabilities(&handle, mode.interface_number, &timeout)?;

        // CLEAR THE BUFFERS AND FEATURES
        // ==========
        control::clear_buffers(&handle, mode.interface_number, &timeout)?;
        control::clear_feature(&handle, &endpoints.bulk_out_ep)?;
        control::clear_feature(&handle, &endpoints.bulk_in_ep)?;

        // RETURN THE CLIENT
        // ==========
        Ok(UsbtmcClient {
            handle,
            mode,
            timeout,
            capabilities,
            btag,
            endpoints,
        })
    }

    /// ### Set Timeout
    ///
    /// Set a new timeout for the device connection.
    ///
    /// #### Arguments
    /// - `duration` -> the duration of the timeout
    ///
    pub fn set_timeout(&self, duration: std::time::Duration) {
        *self.timeout.borrow() = duration;
    }

    /// ### Command
    ///
    /// Send a command to the device.
    ///
    /// #### Arguments
    /// - `cmd` -> the command to send
    ///
    pub fn command(&self, cmd: &str) -> Result<()> {
        use communication::bulk;

        // Send the command
        bulk::write(
            &self.handle,
            &self.btag,
            cmd.into(),
            &self.endpoints.bulk_out_ep,
            &self.timeout,
        )?;

        Ok(())
    }

    /// ### Query Raw
    ///
    /// Send a command and get a response from the device.
    /// The response is a vector of bytes.
    ///
    /// #### Arguments
    /// - `cmd` -> the command to send
    ///
    pub fn query_raw(&self, cmd: &str) -> Result<Vec<u8>> {
        use communication::bulk;

        // Send a command
        bulk::write(
            &self.handle,
            &self.btag,
            cmd.into(),
            &self.endpoints.bulk_out_ep,
            &self.timeout,
        )?;

        // Read the response
        let resp = bulk::read(
            &self.handle,
            &self.btag,
            &self.endpoints.bulk_in_ep,
            &self.endpoints.bulk_out_ep,
            &self.capabilities,
            &self.timeout,
        )?;

        Ok(resp)
    }

    /// ### Query
    ///
    /// Send a command and get a response from the device.
    /// The response is a utf-8 string.
    ///
    /// #### Arguments
    /// - `cmd` -> the command to send
    ///
    pub fn query(&self, cmd: &str) -> Result<String> {
        use communication::bulk;

        // Send a command
        bulk::write(
            &self.handle,
            &self.btag,
            cmd.into(),
            &self.endpoints.bulk_out_ep,
            &self.timeout,
        )?;

        // Read the response
        let resp = bulk::read(
            &self.handle,
            &self.btag,
            &self.endpoints.bulk_in_ep,
            &self.endpoints.bulk_out_ep,
            &self.capabilities,
            &self.timeout,
        )?;

        // Convert response to string
        let resp = std::str::from_utf8(&resp)?.trim();

        Ok(String::from(resp))
    }
}

impl Drop for UsbtmcClient {
    fn drop(&mut self) {
        // RESET THE CONFIGURATION
        // Release the interface
        self.handle
            .borrow()
            .release_interface(self.mode.interface_number)
            .expect("failed to release device usb interface");
        // Reattach the kernel driver if it was disconnected
        if self.mode.has_kernel_driver {
            self.handle
                .borrow()
                .attach_kernel_driver(self.mode.interface_number)
                .expect("failed to attach kernel driver to usb device");
        };
    }
}
