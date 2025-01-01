#[cfg(target_os = "macos")]
use coreaudio::sys::*;
use std::mem::{size_of, size_of_val};
use std::ptr::null;
use std::os::raw::c_void;

#[cfg(target_os = "macos")]
pub fn get_default_output_device() -> Result<AudioDeviceID, OSStatus> {
    let system_object = kAudioObjectSystemObject;

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut device_id: AudioDeviceID = kAudioObjectUnknown;
    let mut size: u32 = size_of::<AudioDeviceID>() as u32;

    let status = unsafe {
        AudioObjectGetPropertyData(
            system_object,
            &property_address as *const AudioObjectPropertyAddress,
            0,
            null(),
            &mut size,
            &mut device_id as *mut AudioDeviceID as *mut c_void,
        )
    };

    if status == 0 {
        Ok(device_id)
    } else {
        Err(status)
    }
}

#[cfg(target_os = "macos")]
pub fn get_device_volume(device_id: AudioDeviceID) -> Result<f32, OSStatus> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwareServiceDeviceProperty_VirtualMasterVolume,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let mut volume: f32 = 0.0;
    let mut size: u32 = size_of::<f32>() as u32;

    let status = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &property_address as *const AudioObjectPropertyAddress,
            0,
            null(),
            &mut size,
            &mut volume as *mut f32 as *mut c_void,
        )
    };

    if status == 0 {
        Ok(volume)
    } else {
        Err(status)
    }
}

#[cfg(target_os = "macos")]
pub fn set_device_volume(device_id: AudioDeviceID, volume: f32) -> Result<(), OSStatus> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwareServiceDeviceProperty_VirtualMasterVolume,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let status = unsafe {
        AudioObjectSetPropertyData(
            device_id,
            &property_address as *const AudioObjectPropertyAddress,
            0,
            null(),
            size_of::<f32>() as u32,
            &volume as *const f32 as *const c_void,
        )
    };

    if status == 0 {
        Ok(())
    } else {
        Err(status)
    }
}

// No-op implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn get_default_output_device() -> Result<u32, i32> {
    Ok(0)
}

#[cfg(not(target_os = "macos"))]
pub fn get_device_volume(_device_id: u32) -> Result<f32, i32> {
    Ok(1.0)
}

#[cfg(not(target_os = "macos"))]
pub fn set_device_volume(_device_id: u32, _volume: f32) -> Result<(), i32> {
    Ok(())
}