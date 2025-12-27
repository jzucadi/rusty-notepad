#[cfg(target_os = "macos")]
mod macos {
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;
    use std::os::raw::c_char;

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOServiceMatching(name: *const c_char) -> *mut std::ffi::c_void;
        fn IOServiceGetMatchingServices(
            mainPort: u32,
            matching: *mut std::ffi::c_void,
            existing: *mut u32,
        ) -> i32;
        fn IOIteratorNext(iterator: u32) -> u32;
        fn IORegistryEntryCreateCFProperties(
            entry: u32,
            properties: *mut *mut std::ffi::c_void,
            allocator: *const std::ffi::c_void,
            options: u32,
        ) -> i32;
        fn IOObjectRelease(object: u32) -> i32;
    }

    const KERN_SUCCESS: i32 = 0;

    pub fn get_gpu_usage() -> Option<f32> {
        unsafe {
            let matching = IOServiceMatching(b"IOAccelerator\0".as_ptr() as *const c_char);
            if matching.is_null() {
                return None;
            }

            let mut iterator: u32 = 0;
            let result = IOServiceGetMatchingServices(0, matching, &mut iterator);
            if result != KERN_SUCCESS {
                return None;
            }

            let mut total_usage: f32 = 0.0;
            let mut gpu_count: u32 = 0;

            loop {
                let entry = IOIteratorNext(iterator);
                if entry == 0 {
                    break;
                }

                let mut properties: *mut std::ffi::c_void = std::ptr::null_mut();
                let result = IORegistryEntryCreateCFProperties(
                    entry,
                    &mut properties,
                    std::ptr::null(),
                    0,
                );

                if result == KERN_SUCCESS && !properties.is_null() {
                    let dict: CFDictionary<CFString, CFType> =
                        CFDictionary::wrap_under_create_rule(properties as *mut _);

                    let perf_key = CFString::new("PerformanceStatistics");
                    if let Some(perf_stats) = dict.find(&perf_key) {
                        let perf_dict: CFDictionary<CFString, CFType> =
                            CFDictionary::wrap_under_get_rule(perf_stats.as_CFTypeRef() as *const _);

                        // Try different keys that might contain GPU utilization
                        let utilization_keys = [
                            "Device Utilization %",
                            "GPU Activity(%)",
                            "GPU Core Utilization",
                            "hardwareWaitTime",
                        ];

                        for key_str in &utilization_keys {
                            let key = CFString::new(key_str);
                            if let Some(value) = perf_dict.find(&key) {
                                if let Some(num) = value.downcast::<CFNumber>() {
                                    if let Some(usage) = num.to_f32() {
                                        total_usage += usage;
                                        gpu_count += 1;
                                        break;
                                    } else if let Some(usage) = num.to_i64() {
                                        total_usage += usage as f32;
                                        gpu_count += 1;
                                        break;
                                    }
                                }
                            }
                        }
                    }
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
}

#[cfg(target_os = "macos")]
pub use macos::get_gpu_usage;

#[cfg(not(target_os = "macos"))]
pub fn get_gpu_usage() -> Option<f32> {
    None
}
