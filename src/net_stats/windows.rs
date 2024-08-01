use std::collections::HashMap;
use std::io;
use std::slice;
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, FALSE, NO_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{GetIfTable, MIB_IFTABLE};

pub fn get_net_dev_stats() -> std::result::Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let mut size = 0;

    unsafe {
        let result = GetIfTable(None, &mut size, FALSE);
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
        let result = GetIfTable(Some(table), &mut size, FALSE);
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
