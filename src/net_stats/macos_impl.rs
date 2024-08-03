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

        // Get a map of interface indices to names
        let iface_names = get_interface_names_by_index()?;

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
                if let Some(name) = iface_names.get(&(if2.ifm_index as u32)) {
                    let bytes_in = data.ifi_ibytes;
                    let bytes_out = data.ifi_obytes;

                    // Insert the interface name and its statistics into the index map
                    index_map.insert(name.clone(), (bytes_in, bytes_out));
                }
            }
        }

        Ok(index_map)
    }
}

/// Fetch a map of interface indices to their names.
///
/// Returns an `IndexMap` where the key is the interface index and the value is the interface name.
unsafe fn get_interface_names_by_index() -> Result<IndexMap<u32, String>, Error> {
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

    let mut iface_names = IndexMap::new();

    // Iterate over the buffer to extract interface indices and names
    while next < end {
        let ifm = next as *const libc::if_msghdr;
        next = next.add((*ifm).ifm_msglen as usize);

        // Check if the message type is RTM_IFINFO2 (network interface info)
        if (*ifm).ifm_type as i32 == libc::RTM_IFINFO2 {
            let if2 = &*(ifm as *const libc::if_msghdr2);

            let sdl = (if2 as *const libc::if_msghdr2).offset(1) as *const libc::sockaddr_dl;
            let sdl_name = (*sdl).sdl_data.as_ptr();
            let sdl_nlen = (*sdl).sdl_nlen as usize;
            let name_slice = std::slice::from_raw_parts(sdl_name as *const u8, sdl_nlen);

            if let Ok(name) = std::str::from_utf8(name_slice) {
                iface_names.insert(if2.ifm_index as u32, name.to_string());
            }
        }
    }

    Ok(iface_names)
}

pub fn get_device_string_to_name_map() -> IndexMap<String, String> {
    IndexMap::new() // there are no crazy iface names, i think
}
