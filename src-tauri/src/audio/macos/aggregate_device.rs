use core_foundation::{
    array::CFArray, base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
  };
  use coreaudio_sys::{
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceMainSubDeviceKey,
    kAudioAggregateDeviceNameKey, kAudioAggregateDeviceSubDeviceListKey,
    kAudioAggregateDeviceTapAutoStartKey, kAudioAggregateDeviceTapListKey,
    kAudioAggregateDeviceUIDKey, kAudioSubDeviceUIDKey, kAudioSubTapDriftCompensationKey,
    kAudioSubTapUIDKey, AudioHardwareCreateAggregateDevice, AudioHardwareDestroyAggregateDevice,
    AudioObjectID, CFDictionaryRef,
  };
  use log;
  use objc_foundation::NSString;
  use objc_id::Id;
  use std::ffi::CStr;

  use super::{ca_tap_description::CATapDescription, tap::audio_hardware_create_process_tap};

  fn cfstring_from_bytes_with_nul(bytes: &'static [u8]) -> CFString {
    let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(bytes) };
    CFString::new(cstr.to_str().unwrap())
  }

  fn uuid_nsstring_to_cfstring(uuid_nsstring: Id<NSString>) -> CFString {
    unsafe {
      let raw_ptr: *const NSString = &*uuid_nsstring;
      CFString::wrap_under_get_rule(raw_ptr as *const _)
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
    log::info!(
      "Creating aggregate device with input_uid: {} output_uid: {} name: {} uid: {}",
      input_uid, output_uid, aggregate_device_name, aggregate_device_uid
    );

    // Create the tap description
    let tap_description = CATapDescription::new_stereo_global_tap_but_exclude();
    let tap_id =
      audio_hardware_create_process_tap(&tap_description).expect("Failed to create tap");

    // Prepare CFStrings
    let aggregate_device_name = CFString::new(aggregate_device_name);
    let aggregate_device_uid = CFString::new(aggregate_device_uid);
    let input_uid_cfstr = CFString::new(input_uid);
    let output_uid_cfstr = CFString::new(output_uid);

    // Create dictionaries for both input and output devices
    let input_device_dict = CFDictionary::from_CFType_pairs(&[(
      cfstring_from_bytes_with_nul(kAudioSubDeviceUIDKey).as_CFType(),
      input_uid_cfstr.as_CFType(),
    )]);

    let output_device_dict = CFDictionary::from_CFType_pairs(&[(
      cfstring_from_bytes_with_nul(kAudioSubDeviceUIDKey).as_CFType(),
      output_uid_cfstr.as_CFType(),
    )]);

    // Create sub-device list
    let sub_device_list = CFArray::from_CFTypes(&[input_device_dict, output_device_dict]);

    // Create tap list
    let tap_uuid_string = uuid_nsstring_to_cfstring(tap_description.get_uuid());
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

    let aggregate_device_description = description_dict.as_concrete_TypeRef() as CFDictionaryRef;

    let mut aggregate_device_id: AudioObjectID = 0;

    let status = unsafe {
      AudioHardwareCreateAggregateDevice(aggregate_device_description, &mut aggregate_device_id)
    };

    if status == 0 {
      log::info!(
        "Created aggregate device {} with tap {}",
        aggregate_device_id, tap_id
      );
      Ok(CreateAggregateDeviceResult {
        aggregate_device_id,
        tap_id,
      })
    } else {
      log::info!(
        "AudioHardwareCreateAggregateDevice failed with status: {}",
        coreaudio::Error::from_os_status(status).unwrap_err()
      );
      Err(coreaudio::Error::from_os_status(status).unwrap_err())
    }
  }

  pub fn remove_aggregate_device(device_id: AudioObjectID) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Removing aggregate device {}", device_id);

    unsafe {
      let result = AudioHardwareDestroyAggregateDevice(device_id);
      if result != 0 {
        return Err(format!("Failed to destroy aggregate device: {}", result).into());
      }
    }

    Ok(())
  }