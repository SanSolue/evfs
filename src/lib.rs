/*
EVFS: Encryptable Virtual File System

A simple file system abstraction for rust game engines and applications.

@author: Samith Sandayake <samith@sansolue.com>
@license: LGPL-3.0-or-later
@version: 0.1.0

*/

mod core;

#[cfg(feature = "local")]
mod local;

#[cfg(feature = "local_enc")]
mod local_encrypted;

pub use core::*;

#[cfg(feature = "local")]
pub use local::*;

#[cfg(feature = "local_enc")]
pub use local_encrypted::*;