// Copyright(c) 2019 Pierre Krieger

use parity_scale_codec::{Decode, Encode};

// TODO: this has been randomly generated; instead should be a hash or something
pub const INTERFACE: [u8; 32] = [
    0x49, 0x6e, 0x56, 0x14, 0x8c, 0xd4, 0x2b, 0xc3, 0x9b, 0x4e, 0xbf, 0x5e, 0xb6, 0x2c, 0x60, 0x4d,
    0x7d, 0xd5, 0x70, 0x92, 0x4d, 0x4f, 0x70, 0xdf, 0xb3, 0xda, 0xf6, 0xfe, 0xdc, 0x65, 0x93, 0x8a,
];

#[derive(Debug, Encode, Decode)]
pub enum InterfaceMessage {
    Register([u8; 32]),
}

#[derive(Debug, Encode, Decode)]
pub struct InterfaceRegisterResponse {
    pub result: Result<(), InterfaceRegisterError>,
}

#[derive(Debug, Encode, Decode)]
pub enum InterfaceRegisterError {
    /// There already exists a process registered for this interface.
    AlreadyRegistered,
}
