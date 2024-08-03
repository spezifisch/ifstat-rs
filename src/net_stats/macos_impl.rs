use indexmap::IndexMap;
use libc::{c_int, c_void, sysctl, CTL_NET, NET_RT_IFLIST2, PF_ROUTE};
use std::io::Error;
use std::ptr::null_mut;

pub fn get_net_dev_stats() -> Result<IndexMap<String, (u64, u64)>, Error> {
    unsafe {
        let mut mib: [c_int; 6] = [CTL_NET, PF_ROUTE, 0, 0, NET_RT_IFLIST2, 0];
        let mut len: usize = 0;

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

        let mut buf = vec![0u8; len];
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

        while next < end {
            let ifm = next as *const libc::if_msghdr2;
            next = next.add((*ifm).ifm_msglen as usize);

            if (*ifm).ifm_type as i32 == libc::RTM_IFINFO2 {
                let if2 = &*(ifm as *const libc::if_msghdr2);
                let data = &if2.ifm_data;

                let name = get_interface_name(if2.ifm_index as u32)?;

                let bytes_in = data.ifi_ibytes;
                let bytes_out = data.ifi_obytes;

                index_map.insert(name, (bytes_in, bytes_out));
            }
        }

        Ok(index_map)
    }
}

unsafe fn get_interface_name(index: u32) -> Result<String, Error> {
    let mut mib: [c_int; 6] = [CTL_NET, PF_ROUTE, 0, 0, NET_RT_IFLIST2, 0];
    let mut len: usize = 0;

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

    let mut buf = vec![0u8; len];
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

    while next < end {
        let ifm = next as *const libc::if_msghdr;
        next = next.add((*ifm).ifm_msglen as usize);

        if (*ifm).ifm_type as i32 == libc::RTM_IFINFO2 {
            let if2 = &*(ifm as *const libc::if_msghdr2);

            if if2.ifm_index as u32 == index {
                let sdl = (if2 as *const libc::if_msghdr2).offset(1) as *const libc::sockaddr_dl;
                let sdl_name = (*sdl).sdl_data.as_ptr() as *const i8;
                let sdl_nlen = (*sdl).sdl_nlen as usize;
                let name_slice = std::slice::from_raw_parts(sdl_name as *const u8, sdl_nlen);
                let name = std::str::from_utf8(name_slice).unwrap().to_string();
                return Ok(name);
            }
        }
    }

    Err(Error::new(
        std::io::ErrorKind::NotFound,
        "Interface name not found",
    ))
}

pub fn get_device_string_to_name_map() -> IndexMap<String, String> {
    let device_string_map = IndexMap::new();

    device_string_map
}
