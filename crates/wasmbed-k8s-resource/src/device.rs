use chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use wasmbed_types::PublicKey;

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    JsonSchema,
    CustomResource,
)]
#[kube(
    group = "wasmbed.github.io",
    version = "v0",
    kind = "Device",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSpec {
    public_key: PublicKey<'static>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DeviceStatus {
    /// Current device phase
    #[serde(default)]
    phase: DevicePhase,

    /// Gateway pod name the device is connected to
    #[serde(skip_serializing_if = "Option::is_none")]
    gateway: Option<String>,

    /// Connection establishment timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    connected_since: Option<DateTime<Utc>>,

    /// Last heartbeat timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    last_heartbeat: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
#[serde(rename_all = "PascalCase")]
pub enum DevicePhase {
    #[default]
    Pending,
    Connected,
    Disconnected,
}
