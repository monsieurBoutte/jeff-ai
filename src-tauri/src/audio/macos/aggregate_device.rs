use core_foundation::{
  array::CFArray, base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
};
use coreaudio_sys::{
  kAudioAggregateDeviceIsPrivateKey,
  kAudioAggregateDeviceMainSubDeviceKey,
  kAudioAggregateDeviceMasterSubDeviceKey,
  kAudioAggregateDeviceNameKey,
  kAudioAggregateDeviceSubDeviceListKey,
  kAudioAggregateDeviceTapAutoStartKey,
  kAudioAggregateDeviceTapListKey,
  kAudioAggregateDeviceUIDKey,
  kAudioSubDeviceUIDKey,
  kAudioSubTapDriftCompensationKey,
  kAudioSubTapUIDKey,
  AudioHardwareCreateAggregateDevice,
  AudioHardwareDestroyAggregateDevice,
  AudioObjectID,
  CFDictionaryRef,
};
use log::{info, error};
use objc_foundation::NSString;
use objc_id::Id;
use std::ffi::CStr;

// Bring in your CATapDescription and audio_hardware_create_process_tap
// from the same module where they were originally.
use super::{
  ca_tap_description::CATapDescription,
  tap::audio_hardware_create_process_tap,
};

/// Convert a `&[u8]` that includes the trailing NUL byte into a `CFString`.
fn cfstring_from_bytes_with_nul(bytes: &'static [u8]) -> CFString {
  let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(bytes) };
  CFString::new(cstr.to_str().unwrap())
}

/// Convert an Objective-C `NSString` (UUID) into a Core Foundation `CFString`.
fn uuid_nsstring_to_cfstring(uuid_nsstring: Id<NSString>) -> CFString {
  unsafe {
      let raw_ptr: *const NSString = &*uuid_nsstring;
      CFString::wrap_under_get_rule(raw_ptr as *const _)
  }
}

/// Return type when creating an aggregate device (includes `tap_id` for output capture).
pub struct CreateAggregateDeviceResult {
  pub tap_id: AudioObjectID,
  pub aggregate_device_id: AudioObjectID,
}

/// Creates a single aggregate device that includes **both** the `input_uid` and `output_uid`.
///
/// - Taps the **output** sub-device so you can capture system audio.
/// - Sets the input sub-device as “master” to keep your mic clock stable (you can swap it if you like).
/// - Marks the aggregator as private (so it doesn’t appear in Audio MIDI Setup). Change to `false` if you want to see it there.
///
pub fn create_aggregate_device(
  input_uid: &str,
  output_uid: &str,
  aggregate_device_name: &str,
  aggregate_device_uid: &str,
) -> Result<CreateAggregateDeviceResult, coreaudio::Error> 
{
  info!(
    "Creating aggregate device with input_uid: {} \
     output_uid: {} name: {} uid: {}",
    input_uid, output_uid, aggregate_device_name, aggregate_device_uid
  );

  // 1) Create a global tap that will let us capture the output device.
  //    You can switch to `new_mono_global_tap_but_exclude()` or
  //    `new_stereo_global_tap_but_exclude()` depending on your needs.
  let tap_description = CATapDescription::new_stereo_global_tap_but_exclude();
  let tap_id = audio_hardware_create_process_tap(&tap_description)
      .expect("Failed to create tap for output aggregator");

  // 2) Convert parameters to CFStrings
  let agg_name_cf = CFString::new(aggregate_device_name);
  let agg_uid_cf = CFString::new(aggregate_device_uid);
  let in_uid_cf = CFString::new(input_uid);
  let out_uid_cf = CFString::new(output_uid);

  // 3) Build sub-device dictionaries for input + output
  let in_dict = CFDictionary::from_CFType_pairs(&[(
      cfstring_from_bytes_with_nul(kAudioSubDeviceUIDKey).as_CFType(),
      in_uid_cf.as_CFType(),
  )]);
  let out_dict = CFDictionary::from_CFType_pairs(&[(
      cfstring_from_bytes_with_nul(kAudioSubDeviceUIDKey).as_CFType(),
      out_uid_cf.as_CFType(),
  )]);

  // Log the contents of in_dict and out_dict for inspection
  log::info!("Input Device Dictionary: {:?}", in_dict);
  log::info!("Output Device Dictionary: {:?}", out_dict);

  let sub_device_list = CFArray::from_CFTypes(&[in_dict, out_dict]);

  // 4) Build the tap list, which references the tap’s unique UUID
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

  // 5) Create the aggregate device dictionary
  //
  //    - The “main” sub-device is set to the output, so that the aggregator will
  //      treat the output device as the default “playback” sub-device.
  //    - The “master” sub-device is set to the input (mic). If you want the output
  //      to drive the clock, you can set `kAudioAggregateDeviceMasterSubDeviceKey` to out_uid_cf.
  //
  //    - We’re also marking it as private with `kAudioAggregateDeviceIsPrivateKey`.
  //      Change to CFBoolean::false_value() if you want to see it in Audio MIDI Setup.
  //
  let description_dict = CFDictionary::from_CFType_pairs(&[
      (
          cfstring_from_bytes_with_nul(kAudioAggregateDeviceNameKey).as_CFType(),
          agg_name_cf.as_CFType(),
      ),
      (
          cfstring_from_bytes_with_nul(kAudioAggregateDeviceUIDKey).as_CFType(),
          agg_uid_cf.as_CFType(),
      ),
      // “Main” sub-device -> output
      (
          cfstring_from_bytes_with_nul(kAudioAggregateDeviceMainSubDeviceKey).as_CFType(),
          out_uid_cf.as_CFType(),
      ),
      // “Master” sub-device -> input
      (
          cfstring_from_bytes_with_nul(kAudioAggregateDeviceMasterSubDeviceKey).as_CFType(),
          in_uid_cf.as_CFType(),
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

  // Convert the dictionary into a CFDictionaryRef for AudioHardwareCreateAggregateDevice
  let aggregate_device_description = description_dict.as_concrete_TypeRef() as CFDictionaryRef;

  // 6) Actually create the aggregate device
  let mut aggregate_device_id: AudioObjectID = 0;
  let status = unsafe {
      AudioHardwareCreateAggregateDevice(
          aggregate_device_description,
          &mut aggregate_device_id
      )
  };

  if status == 0 {
      info!(
          "Successfully created aggregate device {} with tap {}",
          aggregate_device_id,
          tap_id
      );
      Ok(CreateAggregateDeviceResult {
          aggregate_device_id,
          tap_id,
      })
  } else {
      let err = coreaudio::Error::from_os_status(status).unwrap_err();
      error!("AudioHardwareCreateAggregateDevice failed: {}", err);
      Err(err)
  }
}

/// Destroys (removes) the previously created aggregate device.
///
/// Make sure you also stop and clean up any open audio streams referencing it
/// before calling this function.
pub fn remove_aggregate_device(
  device_id: AudioObjectID
) -> Result<(), Box<dyn std::error::Error>> {
  info!("Removing aggregate device {}", device_id);

  unsafe {
      let result = AudioHardwareDestroyAggregateDevice(device_id);
      if result != 0 {
          return Err(format!("Failed to destroy aggregate device: {}", result).into());
      }
  }

  Ok(())
}
