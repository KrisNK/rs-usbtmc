//! ## Initialization
//!
//! A set of functions to help initialize a connection to the device.
//!

use crate::{
    constants::usb::*,
    error::Error,
    types::{DeviceAddr, DeviceId, DeviceInfo, DeviceMode, Endpoint, UsbtmcEndpoints},
    DeviceFilter,
};

use anyhow::Result;
use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, Direction, TransferType, UsbContext};

/// Get first found TMC device
impl DeviceFilter for () {
    fn apply_filter<T: UsbContext>(
        &self,
        _device: &Device<T>,
        _device_desc: &DeviceDescriptor,
    ) -> bool {
        true
    }
}

/// Get TMC device by USB device address
impl DeviceFilter for DeviceAddr {
    fn apply_filter<T: UsbContext>(
        &self,
        device: &Device<T>,
        _device_desc: &DeviceDescriptor,
    ) -> bool {
        self.bus == device.bus_number() && self.device == device.address()
    }
}

/// Get TMC device by USB device address (bus, address)
impl DeviceFilter for (u8, u8) {
    fn apply_filter<T: UsbContext>(
        &self,
        device: &Device<T>,
        _device_desc: &DeviceDescriptor,
    ) -> bool {
        self.0 == device.bus_number() && self.1 == device.address()
    }
}

/// Get TMC device by USB device address [bus, address]
impl DeviceFilter for [u8; 2] {
    fn apply_filter<T: UsbContext>(
        &self,
        device: &Device<T>,
        _device_desc: &DeviceDescriptor,
    ) -> bool {
        self[0] == device.bus_number() && self[1] == device.address()
    }
}

/// Get TMC device by USB identifiers
impl DeviceFilter for DeviceId {
    fn apply_filter<T: UsbContext>(
        &self,
        _device: &Device<T>,
        device_desc: &DeviceDescriptor,
    ) -> bool {
        self.vendor_id == device_desc.vendor_id() && self.product_id == device_desc.product_id()
    }
}

/// Get TMC device by USB identifiers (idVendor, idProduct)
impl DeviceFilter for (u16, u16) {
    fn apply_filter<T: UsbContext>(
        &self,
        _device: &Device<T>,
        device_desc: &DeviceDescriptor,
    ) -> bool {
        self.0 == device_desc.vendor_id() && self.0 == device_desc.product_id()
    }
}

/// Get TMC device by USB identifiers [idVendor, idProduct]
impl DeviceFilter for [u16; 2] {
    fn apply_filter<T: UsbContext>(
        &self,
        _device: &Device<T>,
        device_desc: &DeviceDescriptor,
    ) -> bool {
        self[0] == device_desc.vendor_id() && self[1] == device_desc.product_id()
    }
}

/// Get TMC device by info (both USB identifiers and address)
impl DeviceFilter for DeviceInfo {
    fn apply_filter<T: UsbContext>(
        &self,
        device: &Device<T>,
        device_desc: &DeviceDescriptor,
    ) -> bool {
        self.id.apply_filter(device, device_desc) && self.address.apply_filter(device, device_desc)
    }
}

/// Allow apply filter by reference
impl<T: DeviceFilter> DeviceFilter for &T {
    fn apply_filter<X: UsbContext>(
        &self,
        device: &Device<X>,
        device_desc: &DeviceDescriptor,
    ) -> bool {
        (**self).apply_filter(device, device_desc)
    }
}

/// Allow apply filter by Rc
impl<T: DeviceFilter> DeviceFilter for std::rc::Rc<T> {
    fn apply_filter<X: UsbContext>(
        &self,
        device: &Device<X>,
        device_desc: &DeviceDescriptor,
    ) -> bool {
        (**self).apply_filter(device, device_desc)
    }
}

/// Allow apply filter by Arc
impl<T: DeviceFilter> DeviceFilter for std::sync::Arc<T> {
    fn apply_filter<X: UsbContext>(
        &self,
        device: &Device<X>,
        device_desc: &DeviceDescriptor,
    ) -> bool {
        (**self).apply_filter(device, device_desc)
    }
}

fn is_tmc_device<T: UsbContext>(device: &Device<T>, device_desc: &DeviceDescriptor) -> bool {
    (0..device_desc.num_configurations()).any(move |config_no| {
        if let Ok(config_desc) = device.config_descriptor(config_no) {
            config_desc.interfaces().any(|interface| {
                interface.descriptors().any(|interface_desc| {
                    interface_desc.class_code() == USBTMC_CLASS_CODE
                        && interface_desc.sub_class_code() == USBTMC_SUBCLASS_CODE
                        && interface_desc.protocol_code() == USBTMC_PROTOCOL_CODE
                })
            })
        } else {
            false
        }
    })
}

/// ### List Devices
///
/// List all TMC devices using a libusb context.
///
pub fn list_devices<T: UsbContext>(context: &mut T) -> Result<Vec<DeviceInfo>> {
    Ok(context
        .devices()?
        .iter()
        .filter_map(|device| {
            let device_desc = device.device_descriptor().ok()?;
            if is_tmc_device(&device, &device_desc) {
                Some(DeviceInfo {
                    id: DeviceId {
                        vendor_id: device_desc.vendor_id(),
                        product_id: device_desc.product_id(),
                    },
                    address: DeviceAddr {
                        bus: device.bus_number(),
                        device: device.address(),
                    },
                })
            } else {
                None
            }
        })
        .collect())
}

/// ### Open Device
///
/// Open the device using a libusb context, a vendor id and a product id.
///
pub fn open_device<T: UsbContext>(
    context: &mut T,
    filter: impl DeviceFilter,
) -> Result<(Device<T>, DeviceHandle<T>)> {
    // list the devices
    let devices = context.devices()?;

    // find the one device we want and open it
    for device in devices.iter() {
        // get the descriptor
        if let Ok(device_desc) = device.device_descriptor() {
            // check the IDs
            if is_tmc_device(&device, &device_desc) && filter.apply_filter(&device, &device_desc) {
                // try open the device
                if let Ok(handle) = device.open() {
                    return Ok((device, handle));
                }
            }
        }
    }

    Err(Error::DeviceNotFound.into())
}

/// ### Get USBTMC Mode
///
/// Get the device mode (configuration, interface and interface setting) that is compatible with USBTMC.
///
pub fn get_usbtmc_mode(device: &Device<Context>) -> Result<DeviceMode> {
    // setup the output
    let mut modes: Vec<DeviceMode> = Vec::new();

    // get the device descriptor
    let device_desc = device.device_descriptor()?;

    // go through the configurations
    for n in 0..device_desc.num_configurations() {
        // get the config descriptor
        let config_desc = device.config_descriptor(n)?;
        // println!("{:#?}", config_desc);
        // go through the interfaces
        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                // println!("{:#?}", interface_desc);
                if interface_desc.class_code() == USBTMC_CLASS_CODE
                    && interface_desc.sub_class_code() == USBTMC_SUBCLASS_CODE
                    && interface_desc.protocol_code() == USBTMC_PROTOCOL_CODE
                {
                    // get the data from the mode
                    modes.push(DeviceMode {
                        config_number: config_desc.number(),
                        interface_number: interface_desc.interface_number(),
                        setting_number: interface_desc.setting_number(),
                        has_kernel_driver: false,
                    })
                }
            }
        }
    }

    // Get the first mode
    let mode = match modes.first() {
        Some(m) => m,
        None => return Err(Error::DeviceIncompatible.into()),
    };

    Ok(mode.clone())
}

/// ### Detach Kernel Driver
///
/// If the interface uses a kernel driver, detach it for the duration of the program.
///
pub fn detach_kernel_driver(
    mode: &mut DeviceMode,
    handle: &mut DeviceHandle<Context>,
) -> Result<()> {
    mode.has_kernel_driver = match handle.kernel_driver_active(mode.interface_number) {
        Ok(true) => {
            handle.detach_kernel_driver(mode.interface_number)?;
            true
        }
        _ => false,
    };

    Ok(())
}

/// ### Get Endpoints
///
/// Get a list of endpoints to use
///
pub fn get_endpoints(mode: &DeviceMode, device: &Device<Context>) -> Result<UsbtmcEndpoints> {
    // Endpoints list
    let mut endpoints_list: Vec<Endpoint> = Vec::new();

    // get the config descriptor
    let config_desc = device.config_descriptor(mode.config_number - 1)?;
    // get the interface
    let interface = match config_desc
        .interfaces()
        .find(|inter| inter.number() == mode.interface_number)
    {
        Some(i) => i,
        None => return Err(Error::InterfaceNotFound.into()),
    };
    // get the interface descriptor (setting)
    let interface_desc = match interface
        .descriptors()
        .find(|d| d.setting_number() == mode.setting_number)
    {
        Some(desc) => desc,
        None => return Err(Error::InterfaceSettingNotFound.into()),
    };

    // With the descriptor, we can now iterate through the endpoints
    for endpoint in interface_desc.endpoint_descriptors() {
        endpoints_list.push(Endpoint {
            address: endpoint.address(),
            max_packet_size: endpoint.max_packet_size(),
            transfer_type: endpoint.transfer_type(),
            direction: endpoint.direction(),
        })
    }

    // Go through the list and identify the specific endpoints
    let bulk_out_ep = match endpoints_list
        .iter()
        .find(|ep| ep.transfer_type == TransferType::Bulk && ep.direction == Direction::Out)
    {
        Some(ep) => ep.clone(),
        None => return Err(Error::BulkOutEndpointNotFound.into()),
    };
    let bulk_in_ep = match endpoints_list
        .iter()
        .find(|ep| ep.transfer_type == TransferType::Bulk && ep.direction == Direction::In)
    {
        Some(ep) => ep.clone(),
        None => return Err(Error::BulkInEndpointNotFound.into()),
    };
    let interrupt_ep = match endpoints_list
        .iter()
        .find(|ep| ep.transfer_type == TransferType::Interrupt && ep.direction == Direction::In)
    {
        Some(ep) => Some(ep.clone()),
        None => None,
    };

    Ok(UsbtmcEndpoints {
        bulk_out_ep,
        bulk_in_ep,
        interrupt_ep,
    })
}
