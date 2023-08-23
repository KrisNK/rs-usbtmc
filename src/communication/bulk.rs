//! Bulk
//!
//! Low level functions to read and write data to the bulk endpoints.
//!

use crate::constants::{bulk_msg_id, misc};
use crate::error::Error;
use crate::types::{BTag, Capabilities, Endpoint, Handle, Timeout};

use anyhow::Result;
use rusb::{Direction, TransferType};

/// ### Write
///
/// Write data to the BULK OUT endpoint.
///
pub fn write(
    handle: &Handle,
    btag: &BTag,
    data: Vec<u8>,
    bulk_out_endpoint: &Endpoint,
    timeout: &Timeout,
) -> Result<()> {
    // verify the endpoint is correct
    if bulk_out_endpoint.direction != Direction::Out
        || bulk_out_endpoint.transfer_type != TransferType::Bulk
    {
        return Err(Error::IncorrectEndpoint.into());
    }

    // count the number of transactions to do
    let num_transactions: usize = match data.len() % misc::APPLICATION_BUFFER_SIZE as usize {
        // the data is exactly sized to the application buffer size
        0 => data.len() / misc::APPLICATION_BUFFER_SIZE as usize,
        // the last transaction will be less big than the application buffer size (which is fine)
        _ => data.len() / misc::APPLICATION_BUFFER_SIZE as usize + 1,
    };

    // Seperate the data into transactions to send
    for (transaction_number, transaction) in data
        .chunks(misc::APPLICATION_BUFFER_SIZE as usize)
        .enumerate()
    {
        // setup the header
        let header = device_dependent_msg_out_header(
            btag.get(),
            transaction.len() as u32,
            transaction_number + 1 == num_transactions,
        )?;

        // setup a vector with the header and the transaction data
        let mut data = Vec::from(header);
        data.append(Vec::from(transaction).as_mut());

        // send the transaction in transfers
        for transfer in data.chunks(bulk_out_endpoint.max_packet_size as usize) {
            // add padding to the transfer if required
            let mut transfer = Vec::from(transfer);
            let padding = transfer.len() % 4;
            for _ in 0..padding {
                transfer.push(0x00)
            }

            // execute the transfer
            handle.borrow().write_bulk(
                bulk_out_endpoint.address,
                &transfer,
                timeout.borrow().clone(),
            )?;
        }
    }

    Ok(())
}

pub fn read(
    handle: &Handle,
    btag: &BTag,
    bulk_in_endpoint: &Endpoint,
    bulk_out_endpoint: &Endpoint,
    device_capabilities: &Capabilities,
    timeout: &Timeout,
) -> Result<Vec<u8>> {
    // SETUP
    // ==========

    // verify the endpoints
    if bulk_out_endpoint.direction != Direction::Out
        || bulk_out_endpoint.transfer_type != TransferType::Bulk
    {
        return Err(Error::IncorrectEndpoint.into());
    }
    if bulk_in_endpoint.direction != Direction::In
        || bulk_out_endpoint.transfer_type != TransferType::Bulk
    {
        return Err(Error::IncorrectEndpoint.into());
    }

    // setup the header for the request
    let term_char = match device_capabilities.supports_bulk_in_term_char {
        true => Some(misc::DEFAULT_TERM_CHAR),
        false => None,
    };
    let request_header = request_device_dependent_msg_in_header(
        btag.get(),
        bulk_in_endpoint.max_packet_size as u32,
        term_char,
    )?;

    let mut end_of_message = false;
    let mut output_data: Vec<u8> = Vec::new();

    let mut buffer: Vec<u8> =
        vec![0x00; bulk_in_endpoint.max_packet_size as usize + misc::USBTMC_HEADER_SIZE];

    // READING LOOP
    // ==========

    while !end_of_message {
        // execute the request
        handle.borrow().write_bulk(
            bulk_out_endpoint.address,
            &request_header,
            timeout.borrow().clone(),
        )?;

        // execute the read
        let bytes_read = handle.borrow().read_bulk(
            bulk_in_endpoint.address,
            &mut buffer,
            timeout.borrow().clone(),
        )?;

        // // get the data
        // let mut data: Vec<u8> = buffer[misc::USBTMC_HEADER_SIZE..bytes_read]
        //     .iter()
        //     .filter(|v| **v != 0x00)
        //     .map(|v| *v)
        //     .collect();

        // Add data to the total output
        output_data.append(&mut buffer[misc::USBTMC_HEADER_SIZE..bytes_read].to_vec());

        // check if its the end of the message
        let read_attributes = buffer[8];
        end_of_message = read_attributes & 0b0000_0001 != 0;
    }

    Ok(output_data)
}

pub fn device_dependent_msg_out_header(
    btag: u8,
    transfer_size: u32,
    end_of_message: bool,
) -> Result<[u8; 12]> {
    let mut header: [u8; 12] = [0x00; 12];

    header[0] = bulk_msg_id::DEVICE_DEPENDENT_MSG_OUT;
    header[1] = btag;
    header[2] = !btag;

    let transfer_size: [u8; 4] = transfer_size.to_le_bytes();
    for n in 0..4 {
        header[n + 4] = transfer_size[n];
    }

    if end_of_message {
        header[8] = 0b0000_0001;
    }

    Ok(header)
}

pub fn request_device_dependent_msg_in_header(
    btag: u8,
    transfer_size: u32,
    term_char: Option<u8>,
) -> Result<[u8; 12]> {
    let mut header: [u8; 12] = [0x00; 12];

    header[0] = bulk_msg_id::REQUEST_DEVICE_DEPENDENT_MSG_IN;
    header[1] = btag;
    header[2] = !header[1];

    let transfer_size: [u8; 4] = transfer_size.to_le_bytes();
    for n in 0..4 {
        header[n + 4] = transfer_size[n];
    }

    match term_char {
        Some(tc) => {
            header[8] = 0b0000_0010;
            header[9] = tc;
        }
        None => {}
    }

    Ok(header)
}

pub fn _vendor_specific_out_header(btag: u8, transfer_size: u32) -> Result<[u8; 12]> {
    let mut header: [u8; 12] = [0x00; 12];

    header[0] = bulk_msg_id::VENDOR_SPECIFIC_MSG_OUT;
    header[1] = btag;
    header[2] = !btag;

    let transfer_size: [u8; 4] = transfer_size.to_le_bytes();
    for n in 0..4 {
        header[n + 4] = transfer_size[n];
    }

    Ok(header)
}

pub fn _request_vendor_specific_in_header(btag: u8, transfer_size: u32) -> Result<[u8; 12]> {
    let mut header: [u8; 12] = [0x00; 12];

    header[0] = bulk_msg_id::REQUEST_VENDOR_SPECIFIC_MSG_IN;
    header[1] = btag;
    header[2] = !btag;

    let transfer_size: [u8; 4] = transfer_size.to_le_bytes();
    for n in 0..4 {
        header[n + 4] = transfer_size[n];
    }

    Ok(header)
}
