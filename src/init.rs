//! ## Initialization
//! 
//! A set of functions to help initialize a connection to the device.
//! 

use crate::types::{DeviceMode, Endpoint, UsbtmcEndpoints};
use crate::error::Error;
use crate::constants::usb::*;

use anyhow::Result;
use rusb::{UsbContext, Device, DeviceHandle, Context, TransferType, Direction};

/// ### Open Device
/// 
/// Open the device using a libusb context, a vendor id and a product id.
/// 
pub fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceHandle<T>)> {
    // list the devices
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    // find the one device we want and open it
    for device in devices.iter() {
        // get the descriptor
        let device_desc = match device.device_descriptor() {
            Ok(desc) => desc,
            Err(_) => continue,
        };

        // check the IDs
        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            // open the device
            match device.open() {
                Ok(handle) => return Some((device, handle)),
                Err(_) => continue,
            }
        }
    }

    None
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
pub fn detach_kernel_driver(mode: &mut DeviceMode, handle: &mut DeviceHandle<Context>) -> Result<()> {
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
        None => {
            return Err(Error::InterfaceSettingNotFound.into())
        }
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
    let bulk_out_ep = match endpoints_list.iter().find(|ep| ep.transfer_type == TransferType::Bulk && ep.direction == Direction::Out ) {
        Some(ep) => ep.clone(),
        None => return Err(Error::BulkOutEndpointNotFound.into())
    };
    let bulk_in_ep = match endpoints_list.iter().find(|ep| ep.transfer_type == TransferType::Bulk && ep.direction == Direction::In ) {
        Some(ep) => ep.clone(),
        None => return Err(Error::BulkInEndpointNotFound.into())
    };
    let interrupt_ep = match endpoints_list.iter().find(|ep| ep.transfer_type == TransferType::Interrupt && ep.direction == Direction::In ) {
        Some(ep) => Some(ep.clone()),
        None => None,
    };

    Ok(UsbtmcEndpoints {
        bulk_out_ep,
        bulk_in_ep,
        interrupt_ep,
    })
}