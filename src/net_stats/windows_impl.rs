// network_utils.rs
// This module provides utility functions to retrieve network device statistics
// and map device strings to friendly names on a Windows system using the Win32 API.

use indexmap::IndexMap;
use std::ffi::CStr;
use std::io;
use std::slice;

use widestring::U16CString;
use windows::Win32::{
    Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR},
    NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_INCLUDE_PREFIX, IP_ADAPTER_ADDRESSES_LH,
    },
    NetworkManagement::IpHelper::{GetIfTable, MIB_IFTABLE},
    Networking::WinSock::AF_UNSPEC,
};

#[derive(Debug)]
struct SomeError;

impl From<std::ffi::NulError> for SomeError {
    fn from(_: std::ffi::NulError) -> Self {
        SomeError
    }
}

impl From<std::ffi::FromBytesWithNulError> for SomeError {
    fn from(_: std::ffi::FromBytesWithNulError) -> Self {
        SomeError
    }
}

/// Retrieves network device statistics including received and transmitted bytes.
///
/// # Returns
///
/// A result containing an IndexMap where the keys are the device names and the values are tuples of (received bytes$
/// In case of an error, returns an io::Error.
pub fn get_net_dev_stats() -> std::result::Result<IndexMap<String, (u64, u64)>, std::io::Error> {
    let mut size = 0;

    unsafe {
        // First call to GetIfTable to get the required buffer size
        let result = GetIfTable(None, &mut size, false);
        if result != ERROR_INSUFFICIENT_BUFFER.0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get buffer size for network interface table",
            ));
        }
    }

    // Allocate buffer with the required size
    let mut buffer: Vec<u8> = vec![0; size as usize];
    let table: *mut MIB_IFTABLE = buffer.as_mut_ptr() as *mut MIB_IFTABLE;

    unsafe {
        // Second call to GetIfTable to retrieve the network interface table
        let result = GetIfTable(Some(table), &mut size, false);
        if result != NO_ERROR.0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get network interface table",
            ));
        }

        let table_ref = &*table;
        let mut stats = IndexMap::new();

        // Convert the table to a slice of rows
        let rows = slice::from_raw_parts(table_ref.table.as_ptr(), table_ref.dwNumEntries as usize);

        for row in rows {
            // Convert UTF-16 interface name to String
            let iface_name_utf16: Vec<u16> = row
                .wszName
                .iter()
                .take_while(|&&c| c != 0)
                .cloned()
                .collect();
            let iface_name = String::from_utf16_lossy(&iface_name_utf16)
                .trim()
                .to_string();

            // Retrieve received and transmitted bytes
            let rx_bytes = row.dwInOctets as u64;
            let tx_bytes = row.dwOutOctets as u64;

            stats.insert(iface_name, (rx_bytes, tx_bytes));
        }

        Ok(stats)
    }
}

/// Retrieves a map of network adapter GUIDs to their friendly names.
///
/// # Returns
///
/// A Result containing an IndexMap where the keys are adapter GUIDs and the values are friendly names.
fn get_adapters_map() -> Result<IndexMap<String, String>, SomeError> {
    let mut adapters_map = IndexMap::new();

    unsafe {
        let mut out_buf_len: u32 = 0;
        // First call to GetAdaptersAddresses to get the required buffer size
        GetAdaptersAddresses(
            AF_UNSPEC.0 as u32,
            GAA_FLAG_INCLUDE_PREFIX,
            None,
            Some(std::ptr::null_mut()),
            &mut out_buf_len,
        );

        if out_buf_len == 0 {
            return Ok(adapters_map);
        }

        // Allocate buffer with the required size
        let mut buffer: Vec<u8> = vec![0; out_buf_len as usize];
        let adapter_addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        // Second call to GetAdaptersAddresses to retrieve adapter addresses
        let result = GetAdaptersAddresses(
            AF_UNSPEC.0 as u32,
            GAA_FLAG_INCLUDE_PREFIX,
            None,
            Some(adapter_addresses),
            &mut out_buf_len,
        );

        if result != NO_ERROR.0 {
            return Ok(adapters_map);
        }

        // Iterate over the linked list of adapter addresses
        let mut current_adapter = adapter_addresses;
        while !current_adapter.is_null() {
            let adapter_name_ptr = (*current_adapter).AdapterName.0;
            let adapter_name = CStr::from_ptr(adapter_name_ptr as *const i8)
                .to_str()
                .expect("Invalid UTF-8")
                .to_owned();

            let friendly_name_ptr = (*current_adapter).FriendlyName.0;
            let friendly_name = U16CString::from_ptr_str(friendly_name_ptr)
                .to_string_lossy()
                .to_owned();

            adapters_map.insert(adapter_name, friendly_name);

            current_adapter = (*current_adapter).Next;
        }
    }

    Ok(adapters_map)
}

/// Retrieves a map of device strings to friendly names.
///
/// # Returns
///
/// An IndexMap where the keys are device strings and the values are friendly names.
pub fn get_device_string_to_name_map() -> IndexMap<String, String> {
    let adapters_map = match get_adapters_map() {
        Ok(map) => map,
        Err(_) => return IndexMap::new(), // Return empty map in case of error
    };
    let mut device_string_map = IndexMap::new();

    for (guid, name) in adapters_map {
        let device_string = format!(r"\DEVICE\TCPIP_{}", guid);
        device_string_map.insert(device_string, name);
    }

    device_string_map
}
