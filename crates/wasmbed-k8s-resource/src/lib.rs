mod device;

#[cfg(feature = "client")]
mod device_client;

pub use device::{Device, DeviceSpec, DevicePhase};

#[cfg(feature = "client")]
pub use device_client::DeviceStatusUpdate;
