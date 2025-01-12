use std::{
  mem,
  os::raw::c_void,
  ptr::{self},
};

use coreaudio::audio_unit::macos_helpers::{get_audio_device_ids, get_device_name};
use coreaudio_sys::{
  kAudioObjectPropertyElementMaster, kAudioObjectPropertyScopeGlobal, kAudioTapPropertyFormat,
  AudioDeviceID, AudioObjectGetPropertyData, AudioObjectPropertyAddress,
  AudioStreamBasicDescription,
};

// pub fn get_device_uid(device_id: AudioDeviceID) -> Result<String, coreaudio::Error> {
//   let property_address = AudioObjectPropertyAddress {
//       mSelector: kAudioDevicePropertyDeviceUID,
//       mScope: kAudioDevicePropertyScopeOutput,
//       mElement: kAudioObjectPropertyElementMaster,
//   };

//   macro_rules! try_status_or_return {
//       ($status:expr) => {
//           if $status != kAudioHardwareNoError as i32 {
//               return Err(coreaudio::Error::from_os_status($status).unwrap_err());
//           }
//       };
//   }

//   let device_uid: CFStringRef = null();
//   let data_size = mem::size_of::<CFStringRef>();
//   let c_str = unsafe {
//       let status = AudioObjectGetPropertyData(
//           device_id,
//           &property_address as *const _,
//           0,
//           null(),
//           &data_size as *const _ as *mut _,
//           &device_uid as *const _ as *mut _,
//       );
//       try_status_or_return!(status);

//       let c_string: *const c_char = CFStringGetCStringPtr(device_uid, kCFStringEncodingUTF8);
//       if c_string.is_null() {
//           let status = AudioObjectGetPropertyData(
//               device_id,
//               &property_address as *const _,
//               0,
//               null(),
//               &data_size as *const _ as *mut _,
//               &device_uid as *const _ as *mut _,
//           );
//           try_status_or_return!(status);
//           let mut buf: [i8; 255] = [0; 255];
//           let result = CFStringGetCString(
//               device_uid,
//               buf.as_mut_ptr(),
//               buf.len() as _,
//               kCFStringEncodingUTF8,
//           );
//           if result == 0 {
//               return Err(coreaudio::Error::from_os_status(result.into()).unwrap_err());
//           }
//           let name: &CStr = CStr::from_ptr(buf.as_ptr());
//           return Ok(name.to_str().unwrap().to_owned());
//       }
//       CStr::from_ptr(c_string as *mut _)
//   };
//   Ok(c_str.to_string_lossy().into_owned())
// }

pub fn check_device_exists(target_name: &str) -> bool {
  let device_ids = get_audio_device_ids().expect("failed to get audio device ids");
  for device_id in device_ids {
      if let Ok(name) = get_device_name(device_id) {
          if name == target_name {
              return true;
          }
      }
  }
  false
}

// pub fn all_device_uids() -> Vec<String> {
//   let device_ids = get_audio_device_ids().expect("failed to get audio device ids");
//   let mut uids = Vec::new();

//   for device_id in device_ids {
//       if let Ok(uid) = get_device_uid(device_id) {
//           println!("device uid: {}", uid);
//           uids.push(uid);
//       }
//   }

//   uids
// }

pub fn get_tap_stream_audio_description(
  tap_id: AudioDeviceID,
) -> Result<AudioStreamBasicDescription, coreaudio::Error> {
  let property_address = AudioObjectPropertyAddress {
      mSelector: kAudioTapPropertyFormat,
      mScope: kAudioObjectPropertyScopeGlobal,
      mElement: kAudioObjectPropertyElementMaster,
  };

  let mut size: u32 = mem::size_of::<AudioStreamBasicDescription>() as u32;
  let mut format = AudioStreamBasicDescription {
      mSampleRate: 0.0,
      mFormatID: 0,
      mFormatFlags: 0,
      mBytesPerPacket: 0,
      mFramesPerPacket: 0,
      mBytesPerFrame: 0,
      mChannelsPerFrame: 0,
      mBitsPerChannel: 0,
      mReserved: 0,
  };

  let status = unsafe {
      AudioObjectGetPropertyData(
          tap_id,
          &property_address,
          0,
          ptr::null(),
          &mut size,
          &mut format as *mut _ as *mut c_void,
      )
  };

  if status == 0 {
      Ok(format)
  } else {
      Err(coreaudio::Error::from_os_status(status).unwrap_err())
  }
}

pub fn set_default_device(device_name: &str) -> Result<(), String> {
    let device_ids = get_audio_device_ids().map_err(|e| e.to_string())?;
    let device_id = device_ids
        .into_iter()
        .find(|&id| get_device_name(id).map(|name| name == device_name).unwrap_or(false))
        .ok_or_else(|| format!("Device '{}' not found", device_name))?;

    unsafe {
        let property_address = coreaudio_sys::AudioObjectPropertyAddress {
            mSelector: coreaudio_sys::kAudioHardwarePropertyDefaultOutputDevice,
            mScope: coreaudio_sys::kAudioObjectPropertyScopeGlobal,
            mElement: coreaudio_sys::kAudioObjectPropertyElementMaster,
        };
        let result = coreaudio_sys::AudioObjectSetPropertyData(
            1,  // kAudioObjectSystemObject
            &property_address,
            0,
            std::ptr::null(),
            std::mem::size_of::<u32>() as u32,
            &device_id as *const _ as *const std::ffi::c_void,
        );
        if result != 0 {
            return Err(format!("Failed to set default device. Error code: {}", result));
        }
        Ok(())
    }
}