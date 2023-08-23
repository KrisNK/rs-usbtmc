//! ## Control
//!
//! Set of control requests to send to the device.
//!

use crate::constants::control_requests::READ_STATUS_BYTE;
use crate::constants::{control_requests, usbtmc_status};
use crate::error::Error;
use crate::types::{Capabilities, Endpoint, Handle, Timeout, CtlBTag};

use anyhow::Result;
use rusb::{Direction, TransferType};

pub fn get_capabilities(
    handle: &Handle,
    interface_number: u8,
    timeout: &Timeout,
) -> Result<Capabilities> {
    // setup the request
    let bm_request_type: u8 = rusb::request_type(
        rusb::Direction::In,
        rusb::RequestType::Class,
        rusb::Recipient::Interface,
    );
    let b_request: u8 = control_requests::GET_CAPABILITIES;
    let w_value: u16 = 0x0000;
    let w_index: u16 = u16::from_le_bytes([interface_number, 0x00]);
    let mut buffer: [u8; 0x0018] = [0x00; 0x0018];

    // execute the request
    handle.borrow().read_control(
        bm_request_type,
        b_request,
        w_value,
        w_index,
        &mut buffer,
        timeout.borrow().clone(),
    )?;

    // verify the status
    let status = buffer[0];
    match status {
        usbtmc_status::STATUS_SUCCESS => {}
        _ => return Err(Error::StatusUnexpectedFailure.into()),
    };

    // get the bcd_version
    let bcd_version: u16 = u16::from_le_bytes([buffer[2], buffer[3]]);

    // get the capabilities from the buffer
    let interface_capabilities = buffer[4];
    let device_capabilities = buffer[5];

    let accepts_indicator_pulse_request: bool = interface_capabilities & 0b0000_0100 != 0;
    let is_talk_only: bool = interface_capabilities & 0b0000_0010 != 0;
    let is_listen_only: bool = interface_capabilities & 0b0000_0001 != 0;
    let supports_bulk_in_term_char: bool = device_capabilities & 0b0000_0001 != 0;

    Ok(Capabilities {
        bcd_version,
        accepts_indicator_pulse_request,
        is_talk_only,
        is_listen_only,
        supports_bulk_in_term_char,
    })
}

/// ### Abort Bulk Out Transfer
///
/// Abort a transfer on the bulk out endpoint.
///
/// #### Arguments
/// - `handle` -> the device handle to the USB device
/// - `bulk_out_endpoint` - the endpoint for the BULK OUT endpoint
/// - `transfer_btag` -> the btag of the transfer to abort
/// - `timeout` -> the timeout to use for requests
///
/// #### Returns
/// Returns the number of bytes the device read before aborting the transfer
///
pub fn _abort_bulk_out_transfer(
    handle: &Handle,
    bulk_out_endpoint: &Endpoint,
    transfer_btag: u8,
    timeout: &Timeout,
) -> Result<usize> {
    // INITIATE
    // ==========

    // verify the endpoint is correct
    if bulk_out_endpoint.direction != Direction::Out
        || bulk_out_endpoint.transfer_type != TransferType::Bulk
    {
        return Err(Error::IncorrectEndpoint.into());
    }

    // setup the request
    let bm_request_type = rusb::request_type(
        Direction::In,
        rusb::RequestType::Class,
        rusb::Recipient::Endpoint,
    );
    let b_request = control_requests::INITIATE_ABORT_BULK_OUT;
    let w_value = u16::from_le_bytes([0x00, transfer_btag]);
    let w_index = u16::from_le_bytes([0x00, 0b0000_0000 | bulk_out_endpoint.address]);
    let mut buffer: [u8; 0x0002] = [0x00; 0x0002];

    // execute the command
    handle.borrow().read_control(
        bm_request_type,
        b_request,
        w_value,
        w_index,
        &mut buffer,
        timeout.borrow().clone(),
    )?;

    // check the status
    let status = buffer[0];
    match status {
        usbtmc_status::STATUS_SUCCESS => {}
        usbtmc_status::STATUS_FAILED => return Err(Error::StatusFailure.into()),
        usbtmc_status::STATUS_TRANSFER_NOT_IN_PROGRESS => {
            return Err(Error::StatusNoTransferInProgress.into())
        }
        _ => return Err(Error::StatusUnexpectedFailure.into()),
    };

    // CHECK STATUS
    // ==========

    // setup request
    let b_request: u8 = control_requests::CHECK_ABORT_BULK_OUT_STATUS;
    let w_value: u16 = 0x0000;
    let mut buffer: [u8; 0x0008] = [0x00; 0x0008];

    // loop until it isn't pending
    loop {
        handle.borrow().read_control(
            bm_request_type,
            b_request,
            w_value,
            w_index,
            &mut buffer,
            timeout.borrow().clone(),
        )?;
        let status = buffer[0];
        match status {
            usbtmc_status::STATUS_PENDING => continue,
            usbtmc_status::STATUS_SUCCESS => break,
            _ => return Err(Error::StatusUnexpectedFailure.into()),
        }
    }

    // get the bytes that the device received and did NOT discard
    let bytes_read = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;

    Ok(bytes_read)
}

/// ### Abort Bulk In Transfer
///
/// Abort a transfer on the bulk in endpoint.
///
/// #### Arguments
/// - `handle` -> the device handle to the USB device
/// - `bulk_in_endpoint` - the endpoint for the BULK IN endpoint
/// - `transfer_btag` -> the btag of the transfer to abort
/// - `timeout` -> the timeout to use for requests
///
/// #### Returns
/// Returns the number of bytes the device transfered to the host
///
pub fn _abort_bulk_in_transfer(
    handle: &Handle,
    bulk_in_endpoint: &Endpoint,
    transfer_btag: u8,
    timeout: &Timeout,
) -> Result<usize> {
    // INITIATE
    // ==========

    // verify the endpoint is correct
    if bulk_in_endpoint.direction != Direction::In
        || bulk_in_endpoint.transfer_type != TransferType::Bulk
    {
        return Err(Error::IncorrectEndpoint.into());
    }

    // setup the request
    let bm_request_type = rusb::request_type(
        Direction::In,
        rusb::RequestType::Class,
        rusb::Recipient::Endpoint,
    );
    let b_request = control_requests::INITIATE_ABORT_BULK_IN;
    let w_value = u16::from_le_bytes([0x00, transfer_btag]);
    let w_index = u16::from_le_bytes([0x00, 0b1000_0000 | bulk_in_endpoint.address]);
    let mut buffer: [u8; 0x0002] = [0x00; 0x0002];

    // execute the command
    handle.borrow().read_control(
        bm_request_type,
        b_request,
        w_value,
        w_index,
        &mut buffer,
        timeout.borrow().clone(),
    )?;

    // check the status
    let status = buffer[0];
    match status {
        usbtmc_status::STATUS_SUCCESS => {}
        usbtmc_status::STATUS_FAILED => return Err(Error::StatusFailure.into()),
        usbtmc_status::STATUS_TRANSFER_NOT_IN_PROGRESS => {
            return Err(Error::StatusNoTransferInProgress.into())
        }
        _ => return Err(Error::StatusUnexpectedFailure.into()),
    };

    // CHECK STATUS
    // ==========

    // setup request
    let b_request: u8 = control_requests::CHECK_ABORT_BULK_IN_STATUS;
    let w_value: u16 = 0x0000;
    let mut buffer: [u8; 0x0008] = [0x00; 0x0008];

    // loop until it isn't pending
    loop {
        handle.borrow().read_control(
            bm_request_type,
            b_request,
            w_value,
            w_index,
            &mut buffer,
            timeout.borrow().clone(),
        )?;
        let status = buffer[0];
        match status {
            usbtmc_status::STATUS_PENDING => {
                // check if the Bulk IN FIFO is filled or not
                let fifo_is_empty: bool = buffer[1] ^ 0b0000_0001 == 0;
                if !fifo_is_empty {
                    return Err(Error::BulkInFIFONotEmpty.into());
                }
                continue;
            }
            usbtmc_status::STATUS_SUCCESS => break,
            _ => return Err(Error::StatusUnexpectedFailure.into()),
        }
    }

    // get the bytes that the device received and did NOT discard
    let bytes_transfered =
        u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;

    Ok(bytes_transfered)
}

/// ### Clear Buffers
///
/// Clear all input and output buffers associated to the device.
///
/// **WARNING: must abort all BULK transfers and prevent new ones before using this command.**
///
/// #### Arguments
/// - `handle` -> the device handle to the USB device
/// - `interface_number` - the number of the interface to clear
/// - `timeout` -> the timeout to use for requests
///
pub fn clear_buffers(handle: &Handle, interface_number: u8, timeout: &Timeout) -> Result<()> {
    // INTIATE CLEAR
    // ==========

    // setup the request
    let bm_request_type: u8 = rusb::request_type(
        Direction::In,
        rusb::RequestType::Class,
        rusb::Recipient::Interface,
    );
    let b_request: u8 = control_requests::INITIATE_CLEAR;
    let w_value: u16 = 0x0000;
    let w_index: u16 = u16::from_le_bytes([interface_number, 0x00]);
    let mut buffer: [u8; 0x0001] = [0x00; 0x0001];

    // execute the request
    handle.borrow().read_control(
        bm_request_type,
        b_request,
        w_value,
        w_index,
        &mut buffer,
        timeout.borrow().clone(),
    )?;

    let status = buffer[0];
    match status {
        usbtmc_status::STATUS_SUCCESS => {}
        _ => return Err(Error::StatusUnexpectedFailure.into()),
    };

    // CHECK CLEAR
    // ==========

    // setup the request
    let b_request = control_requests::CHECK_CLEAR_STATUS;
    let mut buffer: [u8; 0x0002] = [0x00; 0x0002];

    loop {
        // execute the request
        handle.borrow().read_control(
            bm_request_type,
            b_request,
            w_value,
            w_index,
            &mut buffer,
            timeout.borrow().clone(),
        )?;

        let status = buffer[0];
        match status {
            usbtmc_status::STATUS_PENDING => {
                // check if the Bulk IN FIFO is filled or not
                let fifo_is_empty: bool = buffer[1] ^ 0b0000_0001 == 0;
                if !fifo_is_empty {
                    return Err(Error::BulkInFIFONotEmpty.into());
                }
                continue;
            }
            usbtmc_status::STATUS_SUCCESS => break,
            _ => return Err(Error::StatusUnexpectedFailure.into()),
        }
    }

    Ok(())
}

/// ### Clear Feature
///
/// Clear any halt on the specified endpoint.
///
/// #### Arguments
/// - `handle` -> the device handle to the USB device
/// - `endpoint` - the endpoint to clear
///
pub fn clear_feature(handle: &Handle, endpoint: &Endpoint) -> Result<()> {
    handle.borrow().clear_halt(endpoint.address)?;
    Ok(())
}

/// ### Read Status Byte
/// 
/// Read the status byte through the control endpoint.
/// 
/// #### Arguments
/// - `handle` -> the device handle to the USB device
/// 
pub fn read_status_byte(handle: &Handle, interface_number: u8, ctl_btag: &CtlBTag, timeout: &Timeout) -> Result<u8> {
    // setup the request
    let bm_request_type = rusb::request_type(Direction::In, rusb::RequestType::Class, rusb::Recipient::Interface);
    let b_request: u8 = READ_STATUS_BYTE;
    let w_value: u16 = 0x0000_0000_0000_0000 + (ctl_btag.get() as u16);
    let w_index: u16 = u16::from_le_bytes([interface_number, 0x00]);
    let mut buffer: [u8;0x0003] = [0x00;0x0003];

    // send/read the request
    handle.borrow().read_control(bm_request_type, b_request, w_value, w_index, &mut buffer, *timeout.borrow())?;

    // check that it is successful
    match buffer[0] {
        usbtmc_status::STATUS_SUCCESS => Ok(buffer[2]),
        usbtmc_status::STATUS_FAILED => Err(Error::StatusFailure.into()),
        _ => Err(Error::StatusUnexpectedFailure.into()),
    }
}