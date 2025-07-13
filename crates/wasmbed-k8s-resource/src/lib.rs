// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

mod device;

#[cfg(feature = "client")]
mod device_client;

pub use device::{Device, DeviceSpec, DevicePhase};

#[cfg(feature = "client")]
pub use device_client::DeviceStatusUpdate;
