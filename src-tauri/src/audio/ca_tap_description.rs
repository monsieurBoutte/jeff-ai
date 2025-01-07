extern crate objc;
extern crate objc_foundation;
extern crate objc_id;
extern crate uuid;

use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};
use objc_foundation::{INSArray, INSObject, INSString, NSArray, NSString};
use objc_id::{Id, Owned};
use uuid::Uuid;

#[repr(i64)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CATapMuteBehavior {
    Unmuted = 0,
    Muted = 1,
    MutedWhenTapped = 2,
}

pub struct CATapDescription {
    pub obj: Id<Object, Owned>,
}

impl CATapDescription {
    pub fn new_mono_global_tap_but_exclude(processes: Vec<i32>) -> Self {
        unsafe {
            let class = Class::get("CATapDescription").unwrap();
            let obj: *mut Object = msg_send![class, alloc];
            let nsarray =
                NSArray::from_vec(processes.iter().map(|&id| NSNumber::new(id)).collect());
            let obj: *mut Object = msg_send![obj, initMonoGlobalTapButExcludeProcesses: nsarray];
            Self {
                obj: Id::from_ptr(obj),
            }
        }
    }

    pub fn get_uuid(&self) -> Id<NSString> {
        unsafe {
            let nsuuid: *mut Object = msg_send![self.obj, UUID];
            let uuid_string: Id<NSString> = msg_send![nsuuid, UUIDString];
            uuid_string
        }
    }

    pub fn new_stereo_global_tap_but_exclude(exclude_device_uids: Vec<String>) -> Self {
        unsafe {
            let class = Class::get("CATapDescription").unwrap();
            let obj: *mut Object = msg_send![class, alloc];
            let nsarray: Id<NSArray<NSString, Owned>> = NSArray::new();
            let obj: *mut Object = msg_send![obj, initStereoGlobalTapButExcludeProcesses: &*nsarray];
            Self {
                obj: Id::from_ptr(obj),
            }
        }
    }
}
// Wrapper for NSNumber
pub struct NSNumber {
    obj: Id<Object, Owned>,
}

impl NSNumber {
    pub fn new(value: i32) -> Id<NSNumber> {
        unsafe {
            let class = Class::get("NSNumber").unwrap();
            let obj: *mut Object = msg_send![class, numberWithInt: value];
            Id::from_ptr(obj as *mut NSNumber)
        }
    }
}

unsafe impl objc::Message for NSNumber {}

impl INSObject for NSNumber {
    fn class() -> &'static Class {
        Class::get("NSNumber").unwrap()
    }
}

// Custom NSUUID wrapper
pub struct NSUUID {
    obj: Id<Object, Owned>,
}

impl NSUUID {
    pub fn from_uuid(uuid: Uuid) -> Id<NSUUID> {
        let uuid_string = uuid.to_string();
        let nsstring = NSString::from_str(&uuid_string);
        unsafe {
            let class = Class::get("NSUUID").unwrap();
            let obj: *mut Object = msg_send![class, alloc];
            let obj: *mut Object = msg_send![obj, initWithUUIDString: nsstring];
            Id::from_ptr(obj as *mut NSUUID)
        }
    }
}

unsafe impl objc::Message for NSUUID {}

impl INSObject for NSUUID {
    fn class() -> &'static Class {
        Class::get("NSUUID").unwrap()
    }
}