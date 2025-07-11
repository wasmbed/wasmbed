#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "cert")]
mod cert;

#[cfg(feature = "k8s")]
mod k8s;

#[cfg(feature = "cert")]
pub use cert::PublicKey;

#[cfg(feature = "k8s")]
pub use k8s::GatewayReference;
