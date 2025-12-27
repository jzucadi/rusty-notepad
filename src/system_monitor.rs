/// System monitoring module for CPU, GPU, RAM, and temperature.
/// Consolidates all system resource monitoring with platform-specific implementations.

/// Holds all system statistics in one place
#[derive(Debug, Clone, Default)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub gpu_usage: Option<f32>,
    pub ram_usage: f32,
    pub cpu_temp: Option<f32>,
}

#[cfg(target_os = "macos")]
mod macos {
    use super::SystemStats;
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;
    use std::os::raw::c_char;
    use sysinfo::System;

    // Shared IOKit FFI declarations
    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOServiceMatching(name: *const c_char) -> *mut std::ffi::c_void;
        fn IOServiceGetMatchingService(main_port: u32, matching: *mut std::ffi::c_void) -> u32;
        fn IOServiceGetMatchingServices(
            main_port: u32,
            matching: *mut std::ffi::c_void,
            existing: *mut u32,
        ) -> i32;
        fn IOServiceOpen(
            service: u32,
            owning_task: u32,
            conn_type: u32,
            connection: *mut u32,
        ) -> i32;
        fn IOServiceClose(connection: u32) -> i32;
        fn IOIteratorNext(iterator: u32) -> u32;
        fn IORegistryEntryCreateCFProperties(
            entry: u32,
            properties: *mut *mut std::ffi::c_void,
            allocator: *const std::ffi::c_void,
            options: u32,
        ) -> i32;
        fn IOConnectCallStructMethod(
            connection: u32,
            selector: u32,
            input: *const SMCKeyData,
            input_size: usize,
            output: *mut SMCKeyData,
            output_size: *mut usize,
        ) -> i32;
        fn IOObjectRelease(object: u32) -> i32;
        fn mach_task_self() -> u32;
    }

    const KERN_SUCCESS: i32 = 0;

    // ============== GPU Monitoring ==============

    pub fn get_gpu_usage() -> Option<f32> {
        unsafe {
            let matching = IOServiceMatching(b"IOAccelerator\0".as_ptr() as *const c_char);
            if matching.is_null() {
                return None;
            }

            let mut iterator: u32 = 0;
            if IOServiceGetMatchingServices(0, matching, &mut iterator) != KERN_SUCCESS {
                return None;
            }

            let mut total_usage: f32 = 0.0;
            let mut gpu_count: u32 = 0;

            loop {
                let entry = IOIteratorNext(iterator);
                if entry == 0 {
                    break;
                }

                if let Some(usage) = get_gpu_entry_usage(entry) {
                    total_usage += usage;
                    gpu_count += 1;
                }

                IOObjectRelease(entry);
            }

            IOObjectRelease(iterator);

            if gpu_count > 0 {
                Some(total_usage / gpu_count as f32)
            } else {
                None
            }
        }
    }

    unsafe fn get_gpu_entry_usage(entry: u32) -> Option<f32> {
        let mut properties: *mut std::ffi::c_void = std::ptr::null_mut();
        let result =
            IORegistryEntryCreateCFProperties(entry, &mut properties, std::ptr::null(), 0);

        if result != KERN_SUCCESS || properties.is_null() {
            return None;
        }

        let dict: CFDictionary<CFString, CFType> =
            CFDictionary::wrap_under_create_rule(properties as *mut _);

        let perf_key = CFString::new("PerformanceStatistics");
        let perf_stats = dict.find(&perf_key)?;

        let perf_dict: CFDictionary<CFString, CFType> =
            CFDictionary::wrap_under_get_rule(perf_stats.as_CFTypeRef() as *const _);

        const UTILIZATION_KEYS: &[&str] = &[
            "Device Utilization %",
            "GPU Activity(%)",
            "GPU Core Utilization",
            "hardwareWaitTime",
        ];

        for key_str in UTILIZATION_KEYS {
            let key = CFString::new(key_str);
            if let Some(value) = perf_dict.find(&key) {
                if let Some(num) = value.downcast::<CFNumber>() {
                    if let Some(usage) = num.to_f32() {
                        return Some(usage);
                    } else if let Some(usage) = num.to_i64() {
                        return Some(usage as f32);
                    }
                }
            }
        }

        None
    }

    // ============== Temperature Monitoring (SMC) ==============

    #[repr(C)]
    struct SMCKeyData {
        key: u32,
        vers: [u8; 6],
        p_limit_data: [u8; 16],
        key_info: SMCKeyInfoData,
        result: u8,
        status: u8,
        data8: u8,
        data32: u32,
        bytes: [u8; 32],
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct SMCKeyInfoData {
        data_size: u32,
        data_type: u32,
        data_attributes: u8,
    }

    impl Default for SMCKeyData {
        fn default() -> Self {
            Self {
                key: 0,
                vers: [0; 6],
                p_limit_data: [0; 16],
                key_info: SMCKeyInfoData::default(),
                result: 0,
                status: 0,
                data8: 0,
                data32: 0,
                bytes: [0; 32],
            }
        }
    }

    const KERNEL_INDEX_SMC: u32 = 2;
    const SMC_CMD_READ_KEYINFO: u8 = 9;
    const SMC_CMD_READ_BYTES: u8 = 5;

    fn fourcc_to_u32(s: &[u8; 4]) -> u32 {
        ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
    }

    pub fn get_cpu_temperature() -> Option<f32> {
        unsafe {
            let matching = IOServiceMatching(b"AppleSMC\0".as_ptr() as *const c_char);
            if matching.is_null() {
                return None;
            }

            let service = IOServiceGetMatchingService(0, matching);
            if service == 0 {
                return None;
            }

            let mut connection: u32 = 0;
            let result = IOServiceOpen(service, mach_task_self(), 0, &mut connection);
            IOObjectRelease(service);

            if result != 0 {
                return None;
            }

            let temp = read_cpu_temperature(connection);
            IOServiceClose(connection);
            temp
        }
    }

    unsafe fn read_cpu_temperature(connection: u32) -> Option<f32> {
        // Apple Silicon and Intel temperature sensor keys
        const TEMP_KEYS: &[&[u8; 4]] = &[
            b"Tp09", b"Tp01", b"Tp05", b"Tp0D", b"Tp0H", // Apple Silicon
            b"Tp0L", b"Tp0P", b"Tp0X", b"Tp0b",          // Apple Silicon
            b"TC0P", b"TC0C", b"TC1C", b"TC0D", b"TCXC", // Intel
        ];

        let mut temps: Vec<f32> = Vec::new();

        for key in TEMP_KEYS {
            if let Some(t) = read_smc_key(connection, key) {
                // Valid CPU temps: 20-110°C
                if (20.0..110.0).contains(&t) {
                    temps.push(t);
                }
            }
        }

        if temps.is_empty() {
            None
        } else {
            Some(temps.iter().sum::<f32>() / temps.len() as f32)
        }
    }

    unsafe fn read_smc_key(connection: u32, key: &[u8; 4]) -> Option<f32> {
        let mut input = SMCKeyData::default();
        let mut output = SMCKeyData::default();

        // Get key info
        input.key = fourcc_to_u32(key);
        input.data8 = SMC_CMD_READ_KEYINFO;

        let mut output_size = std::mem::size_of::<SMCKeyData>();
        if IOConnectCallStructMethod(
            connection,
            KERNEL_INDEX_SMC,
            &input,
            std::mem::size_of::<SMCKeyData>(),
            &mut output,
            &mut output_size,
        ) != 0
        {
            return None;
        }

        // Read bytes
        input.key_info = output.key_info;
        input.data8 = SMC_CMD_READ_BYTES;

        if IOConnectCallStructMethod(
            connection,
            KERNEL_INDEX_SMC,
            &input,
            std::mem::size_of::<SMCKeyData>(),
            &mut output,
            &mut output_size,
        ) != 0
        {
            return None;
        }

        parse_temperature_value(&output)
    }

    fn parse_temperature_value(data: &SMCKeyData) -> Option<f32> {
        let data_type = data.key_info.data_type;

        match data_type {
            t if t == fourcc_to_u32(b"sp78") => {
                // Signed 7.8 fixed point
                let raw = ((data.bytes[0] as i16) << 8) | (data.bytes[1] as i16);
                Some(raw as f32 / 256.0)
            }
            t if t == fourcc_to_u32(b"flt ") => {
                // Float
                let bytes = [data.bytes[0], data.bytes[1], data.bytes[2], data.bytes[3]];
                Some(f32::from_be_bytes(bytes))
            }
            _ => {
                // Fallback: try simple byte value
                let temp = data.bytes[0] as f32;
                if (0.0..150.0).contains(&temp) {
                    Some(temp)
                } else {
                    None
                }
            }
        }
    }

    // ============== Combined Stats Collection ==============

    pub fn collect_stats(system: &mut System) -> SystemStats {
        system.refresh_cpu_all();
        system.refresh_memory();

        let total_mem = system.total_memory() as f32;
        let used_mem = system.used_memory() as f32;

        SystemStats {
            cpu_usage: system.global_cpu_usage(),
            gpu_usage: get_gpu_usage(),
            ram_usage: if total_mem > 0.0 {
                (used_mem / total_mem) * 100.0
            } else {
                0.0
            },
            cpu_temp: get_cpu_temperature(),
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos::collect_stats;

#[cfg(not(target_os = "macos"))]
pub fn collect_stats(system: &mut sysinfo::System) -> SystemStats {
    system.refresh_cpu_all();
    system.refresh_memory();

    let total_mem = system.total_memory() as f32;
    let used_mem = system.used_memory() as f32;

    SystemStats {
        cpu_usage: system.global_cpu_usage(),
        gpu_usage: None,
        ram_usage: if total_mem > 0.0 {
            (used_mem / total_mem) * 100.0
        } else {
            0.0
        },
        cpu_temp: None,
    }
}
