use std::collections::HashMap;
use std::ffi::CStr;
use std::io;
use std::ptr::null_mut;
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

pub fn get_net_dev_stats() -> std::result::Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let mut size = 0;

    unsafe {
        let result = GetIfTable(None, &mut size, false);
        if result != ERROR_INSUFFICIENT_BUFFER.0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get buffer size for network interface table",
            ));
        }
    }

    let mut buffer: Vec<u8> = vec![0; size as usize];
    let table: *mut MIB_IFTABLE = buffer.as_mut_ptr() as *mut MIB_IFTABLE;

    unsafe {
        let result = GetIfTable(Some(table), &mut size, false);
        if result != NO_ERROR.0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get network interface table",
            ));
        }

        let table_ref = &*table;
        let mut stats = HashMap::new();

        let rows = slice::from_raw_parts(table_ref.table.as_ptr(), table_ref.dwNumEntries as usize);

        for row in rows {
            let iface_name_utf16: Vec<u16> = row
                .wszName
                .iter()
                .take_while(|&&c| c != 0)
                .cloned()
                .collect();
            let iface_name = String::from_utf16_lossy(&iface_name_utf16)
                .trim()
                .to_string();
            let rx_bytes = row.dwInOctets as u64;
            let tx_bytes = row.dwOutOctets as u64;

            stats.insert(iface_name, (rx_bytes, tx_bytes));
        }

        Ok(stats)
    }
}

fn get_adapters_map() -> HashMap<String, String> {
    let mut adapters_map = HashMap::new();

    unsafe {
        let mut out_buf_len: u32 = 0;
        GetAdaptersAddresses(
            AF_UNSPEC.0 as u32,
            GAA_FLAG_INCLUDE_PREFIX,
            None,
            Some(null_mut()),
            &mut out_buf_len,
        );

        if out_buf_len == 0 {
            return adapters_map;
        }

        let mut buffer: Vec<u8> = vec![0; out_buf_len as usize];
        let adapter_addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        let result = GetAdaptersAddresses(
            AF_UNSPEC.0 as u32,
            GAA_FLAG_INCLUDE_PREFIX,
            None,
            Some(adapter_addresses),
            &mut out_buf_len,
        );

        if result != NO_ERROR.0 {
            return adapters_map;
        }

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

    adapters_map
}

pub fn get_device_string_to_name_map() -> HashMap<String, String> {
    let adapters_map = get_adapters_map();
    let mut device_string_map = HashMap::new();

    for (guid, name) in adapters_map {
        let device_string = format!(r"\DEVICE\TCPIP_{{{}}}", guid);
        device_string_map.insert(device_string, name);
    }

    device_string_map
}
