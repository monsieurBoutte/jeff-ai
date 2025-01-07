use core_foundation::{
  array::CFArray, base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
};
use coreaudio_sys::{
  kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceMainSubDeviceKey, kAudioAggregateDeviceNameKey,
  kAudioAggregateDeviceSubDeviceListKey, kAudioAggregateDeviceTapAutoStartKey,
  kAudioAggregateDeviceTapListKey, kAudioAggregateDeviceUIDKey, kAudioSubDeviceUIDKey,
  kAudioSubTapDriftCompensationKey, kAudioSubTapUIDKey, AudioHardwareCreateAggregateDevice,
  AudioObjectID, CFDictionaryRef, AudioHardwareDestroyAggregateDevice,
};
use log::info;
use objc_foundation::NSString;
use objc_id::Id;

use std::ffi::CStr;

use super::{ca_tap_description::CATapDescription, tap::audio_hardware_create_process_tap};

// Convert kAudio* constants to CFString
fn cfstring_from_bytes_with_nul(bytes: &'static [u8]) -> CFString {
  let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(bytes) };
  CFString::new(cstr.to_str().unwrap())
}

fn uuid_nsstring_to_cfstring(uuid_nsstring: Id<NSString>) -> CFString {
  unsafe {
      let raw_ptr: *const NSString = &*uuid_nsstring;
      let cfstring: CFString = TCFType::wrap_under_get_rule(raw_ptr as *const _);
      cfstring
  }
}

pub struct CreateAggregateDeviceResult {
  pub tap_id: AudioObjectID,
  pub aggregate_device_id: AudioObjectID,
}

pub fn create_aggregate_device(
    input_uid: &str,
    output_uid: &str,
    aggregate_device_name: &str,
    aggregate_device_uid: &str,
) -> Result<CreateAggregateDeviceResult, coreaudio::Error> {
    info!(
        "Creating aggregate device with input_uid: {} output_uid: {} name: {} uid: {}",
        input_uid, output_uid, aggregate_device_name, aggregate_device_uid
    );
    let tap_description = CATapDescription::new_stereo_global_tap_but_exclude(vec![]);
    let tap_id = audio_hardware_create_process_tap(&tap_description).expect("Failed to create tap");

    let aggregate_device_name = CFString::new(aggregate_device_name);
    let aggregate_device_uid = CFString::new(aggregate_device_uid);
    let output_uid_cfstr = CFString::new(&output_uid);

    let sub_device_dict = CFDictionary::from_CFType_pairs(&[(
        cfstring_from_bytes_with_nul(kAudioSubDeviceUIDKey).as_CFType(),
        output_uid_cfstr.as_CFType()
    )]);

    let tap_uuid_string = uuid_nsstring_to_cfstring(tap_description.get_uuid());

    info!("tap_uuid_string {}", tap_uuid_string.to_string());

    let tap_device_dict = CFDictionary::from_CFType_pairs(&[
        (
            cfstring_from_bytes_with_nul(kAudioSubTapDriftCompensationKey).as_CFType(),
            CFBoolean::false_value().as_CFType(),
        ),
        (
            cfstring_from_bytes_with_nul(kAudioSubTapUIDKey).as_CFType(),
            tap_uuid_string.as_CFType(),
        ),
    ]);

    // Sub-device list
    let sub_device_list = CFArray::from_CFTypes(&[sub_device_dict]);

    let tap_list = CFArray::from_CFTypes(&[tap_device_dict]);

    // Create the aggregate device description dictionary
    let description_dict = CFDictionary::from_CFType_pairs(&[
        (
            cfstring_from_bytes_with_nul(kAudioAggregateDeviceNameKey).as_CFType(),
            aggregate_device_name.as_CFType(),
        ),
        (
            cfstring_from_bytes_with_nul(kAudioAggregateDeviceUIDKey).as_CFType(),
            aggregate_device_uid.as_CFType(),
        ),
        (
            cfstring_from_bytes_with_nul(kAudioAggregateDeviceMainSubDeviceKey).as_CFType(),
            output_uid_cfstr.as_CFType(),
        ),
        (
            cfstring_from_bytes_with_nul(kAudioAggregateDeviceIsPrivateKey).as_CFType(),
            CFBoolean::true_value().as_CFType(),
        ),
        (
            cfstring_from_bytes_with_nul(kAudioAggregateDeviceTapAutoStartKey).as_CFType(),
            CFBoolean::true_value().as_CFType(),
        ),
        (
            cfstring_from_bytes_with_nul(kAudioAggregateDeviceSubDeviceListKey).as_CFType(),
            sub_device_list.as_CFType(),
        ),
        (
            cfstring_from_bytes_with_nul(kAudioAggregateDeviceTapListKey).as_CFType(),
            tap_list.as_CFType(),
        ),
    ]);

    // Convert the dictionary to CFDictionaryRef
    let aggregate_device_description = description_dict.as_concrete_TypeRef() as CFDictionaryRef;

    // Initialize the aggregate device ID
    let mut aggregate_device_id: AudioObjectID = 0;

    // Call AudioHardwareCreateAggregateDevice
    let status = unsafe {
        AudioHardwareCreateAggregateDevice(aggregate_device_description, &mut aggregate_device_id)
    };

    if status == 0 {
        info!(
            "Created aggregate device {} with tap {}",
            aggregate_device_id, tap_id
        );
        Ok(CreateAggregateDeviceResult {
            aggregate_device_id,
            tap_id,
        })
    } else {
        info!(
            "AudioHardwareCreateAggregateDevice failed with status: {}",
            coreaudio::Error::from_os_status(status).unwrap_err()
        );
        Err(coreaudio::Error::from_os_status(status).unwrap_err())
    }
}

pub fn remove_aggregate_device(device_id: u32) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let result = AudioHardwareDestroyAggregateDevice(device_id);
        if result != 0 {
            return Err(format!("Failed to destroy aggregate device: {}", result).into());
        }
        Ok(())
    }
}