// net_stats.rs
// This file contains functions for fetching network interface statistics on macOS.
// It retrieves the network interface names and their respective incoming and outgoing byte counts.

use indexmap::IndexMap;
use libc::{c_int, c_void, sysctl, CTL_NET, NET_RT_IFLIST2, PF_ROUTE};
use std::io::Error;
use std::ptr::null_mut;

/// Fetch network device statistics for each network interface.
///
/// Returns an `IndexMap` where the key is the interface name and the value is a tuple containing
/// bytes in and bytes out.
pub fn get_net_dev_stats() -> Result<IndexMap<String, (u64, u64)>, Error> {
    unsafe {
        // Define the MIB (Management Information Base) array for fetching network interface data
        let mut mib: [c_int; 6] = [CTL_NET, PF_ROUTE, 0, 0, NET_RT_IFLIST2, 0];
        let mut len: usize = 0;

        // Get the length of the buffer needed to store the data
        if sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            null_mut(),
            &mut len,
            null_mut(),
            0,
        ) < 0
        {
            return Err(Error::last_os_error());
        }

        // Allocate a buffer with the required length
        let mut buf = vec![0u8; len];
        // Fetch the actual network interface data into the buffer
        if sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            &mut len,
            null_mut(),
            0,
        ) < 0
        {
            return Err(Error::last_os_error());
        }

        let mut index_map = IndexMap::new();
        let mut next = buf.as_ptr();
        let end = buf.as_ptr().add(len);

        // Iterate over the buffer to extract network interface statistics
        while next < end {
            let ifm = next as *const libc::if_msghdr2;
            next = next.add((*ifm).ifm_msglen as usize);

            // Check if the message type is RTM_IFINFO2 (network interface info)
            if (*ifm).ifm_type as i32 == libc::RTM_IFINFO2 {
                let if2 = &*(ifm as *const libc::if_msghdr2);
                let data = &if2.ifm_data;

                // Get the interface name by its index
                let name = get_interface_name(if2.ifm_index as u32)?;

                let bytes_in = data.ifi_ibytes;
                let bytes_out = data.ifi_obytes;

                // Insert the interface name and its statistics into the index map
                index_map.insert(name, (bytes_in, bytes_out));
            }
        }

        Ok(index_map)
    }
}

/// Fetch the name of a network interface by its index.
///
/// Returns the name as a `String`.
unsafe fn get_interface_name(index: u32) -> Result<String, Error> {
    // Define the MIB array for fetching network interface list
    let mut mib: [c_int; 6] = [CTL_NET, PF_ROUTE, 0, 0, NET_RT_IFLIST2, 0];
    let mut len: usize = 0;

    // Get the length of the buffer needed to store the interface list
    if sysctl(
        mib.as_mut_ptr(),
        mib.len() as u32,
        null_mut(),
        &mut len,
        null_mut(),
        0,
    ) < 0
    {
        return Err(Error::last_os_error());
    }

    // Allocate a buffer with the required length
    let mut buf = vec![0u8; len];
    // Fetch the actual interface list into the buffer
    if sysctl(
        mib.as_mut_ptr(),
        mib.len() as u32,
        buf.as_mut_ptr() as *mut c_void,
        &mut len,
        null_mut(),
        0,
    ) < 0
    {
        return Err(Error::last_os_error());
    }

    let mut next = buf.as_ptr();
    let end = buf.as_ptr().add(len);

    // Iterate over the buffer to find the interface with the specified index
    while next < end {
        let ifm = next as *const libc::if_msghdr;
        next = next.add((*ifm).ifm_msglen as usize);

        // Check if the message type is RTM_IFINFO2 (network interface info)
        if (*ifm).ifm_type as i32 == libc::RTM_IFINFO2 {
            let if2 = &*(ifm as *const libc::if_msghdr2);

            if if2.ifm_index as u32 == index {
                let sdl = (if2 as *const libc::if_msghdr2).offset(1) as *const libc::sockaddr_dl;
                let sdl_name = (*sdl).sdl_data.as_ptr();
                let sdl_nlen = (*sdl).sdl_nlen as usize;
                let name_slice = std::slice::from_raw_parts(sdl_name as *const u8, sdl_nlen);
                return std::str::from_utf8(name_slice)
                    .map(|s| s.to_string())
                    .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e));
            }
        }
    }

    Err(Error::new(
        std::io::ErrorKind::NotFound,
        "Interface name not found",
    ))
}

/// Function to get a map of device strings to device names.
///
/// Glues with our old Error-less code.
pub fn get_device_string_to_name_map() -> IndexMap<String, String> {
    get_device_string_to_name_map_impl().unwrap_or(IndexMap::new())
}

/// Get a map of device strings to device names.
///
/// Returns an `IndexMap` where the key is the device string and the value is the device name.
pub fn get_device_string_to_name_map_impl() -> Result<IndexMap<String, String>, Error> {
    let mut device_string_map = IndexMap::new();

    unsafe {
        // Define the MIB (Management Information Base) array for fetching network interface data
        let mut mib: [c_int; 6] = [CTL_NET, PF_ROUTE, 0, 0, NET_RT_IFLIST2, 0];
        let mut len: usize = 0;

        // Get the length of the buffer needed to store the data
        if sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            null_mut(),
            &mut len,
            null_mut(),
            0,
        ) < 0
        {
            return Err(Error::last_os_error());
        }

        // Allocate a buffer with the required length
        let mut buf = vec![0u8; len];
        // Fetch the actual network interface data into the buffer
        if sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            &mut len,
            null_mut(),
            0,
        ) < 0
        {
            return Err(Error::last_os_error());
        }

        let mut next = buf.as_ptr();
        let end = buf.as_ptr().add(len);

        // Iterate over the buffer to extract network interface statistics
        while next < end {
            let ifm = next as *const libc::if_msghdr2;
            next = next.add((*ifm).ifm_msglen as usize);

            // Check if the message type is RTM_IFINFO2 (network interface info)
            if (*ifm).ifm_type as i32 == libc::RTM_IFINFO2 {
                let if2 = &*(ifm as *const libc::if_msghdr2);

                // Get the interface name by its index
                let name = match get_interface_name(if2.ifm_index as u32) {
                    Ok(n) => n,
                    Err(_) => continue, // Skip if we can't get the interface name
                };

                // Insert the interface index and its name into the device string map
                device_string_map.insert(if2.ifm_index.to_string(), name);
            }
        }
    }

    Ok(device_string_map)
}
