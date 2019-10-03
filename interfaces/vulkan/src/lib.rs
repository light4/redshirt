// Copyright(c) 2019 Pierre Krieger

//! Vulkan bindings.
//!
//! # How it works
//!
//! This library contains an implementation of the Vulkan API v1.1. The [`vkGetInstanceProcAddr`]
//! function is the entry point of the Vulkan API, according to [the Vulkan specifications]
//! (https://www.khronos.org/registry/vulkan/specs/1.1-extensions/html/vkspec.html).
//!
//! The way this implementation works is by serializing all the Vulkan function calls into a
//! [`VulkanMessage`] enum and sending it to the interface handler. If return type of the function
//! is not `()`, the function waits for the answer to come back before returning.
//!
//! From the point of view of the user of Vulkan, this is all that is needed.
//!
//! # From the point of view of the interface handler
//!
//! On the side of the interface handler, the serialized Vulkan function calls have to be
//! handled. The most straight-forward way to do that is by directly handling the messages and
//! sending back answers.
//!
//! Another possibility, however, is to use the [`VulkanRedirect`] struct. The [`VulkanRedirect`]
//! can leverage another implementation of Vulkan (through a `vkGetInstanceProcAddr` function) and
//! can handle [`VulkanMessage`]s through the [`VulkanRedirect::handle`] method.
//!

use core::{ffi::c_void, mem, ptr};
use parity_scale_codec::{Decode, Encode};
use std::ffi::CStr;

include!(concat!(env!("OUT_DIR"), "/vk.rs"));

// TODO: this has been randomly generated; instead should be a hash or something
pub const INTERFACE: [u8; 32] = [
    0x30, 0xc1, 0xd8, 0x90, 0x74, 0x2f, 0x9b, 0x1a, 0x11, 0xfc, 0xcb, 0x53, 0x35, 0xc0, 0x6f, 0xe6,
    0x5c, 0x82, 0x13, 0xe3, 0xcc, 0x04, 0x7b, 0xb7, 0xf6, 0x88, 0x74, 0x1e, 0x7a, 0xf2, 0x84, 0x75, 
];

#[allow(non_camel_case_types)]
pub type PFN_vkAllocationFunction = extern "system" fn(*mut c_void, usize, usize, SystemAllocationScope) -> *mut c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkReallocationFunction = extern "system" fn(*mut c_void, *mut c_void, usize, usize, SystemAllocationScope) -> *mut c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkFreeFunction = extern "system" fn(*mut c_void, *mut c_void);
#[allow(non_camel_case_types)]
pub type PFN_vkInternalAllocationNotification = extern "system" fn(*mut c_void, usize, InternalAllocationType, SystemAllocationScope) -> *mut c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkInternalFreeNotification = extern "system" fn(*mut c_void, usize, InternalAllocationType, SystemAllocationScope) -> *mut c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkDebugReportCallbackEXT = extern "system" fn(DebugReportFlagsEXT, DebugReportObjectTypeEXT, u64, usize, i32, *const i8, *const i8, *mut c_void) -> Bool32;
#[allow(non_camel_case_types)]
pub type PFN_vkVoidFunction = extern "system" fn() -> ();

/// Leverages an existing Vulkan implementation to handle [`VulkanMessage`]s.
pub struct VulkanRedirect {
    /// How we retrieve instance proc addresses.
    get_instance_proc_addr: extern "system" fn(usize, *const u8) -> PFN_vkVoidFunction,
}

impl VulkanRedirect {
    pub fn new(get_instance_proc_addr: extern "system" fn(usize, *const u8) -> PFN_vkVoidFunction) -> VulkanRedirect {
        VulkanRedirect {
            get_instance_proc_addr,
        }
    }

    /// Handles the given [`VulkanMessage`], optionally producing the answer to send back in
    /// response to this call.
    pub fn handle(message: VulkanMessage) -> Option<Vec<u8>> {
        // TODO: implement, lol
        panic!("{:?}", message);
        //None
    }
}