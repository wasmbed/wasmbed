use std::sync::Arc;
use kube::{CustomResource, Error};
use kube::runtime::controller::Action;
use kube::client::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use wasmbed_types::DeviceId;

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
    version = "v1",
    kind = "Device",
    namespaced,
)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSpec {
    id: DeviceId,
}

pub async fn reconcile_device(
    _device: Arc<Device>,
    _ctx: Arc<Client>,
) -> Result<Action, Error> {
    Ok(Action::await_change())
}

pub fn on_reconcile_device_error(
    device: Arc<Device>,
    error: &kube::Error,
    _ctx: Arc<Client>
) -> Action {
    println!(
        "Reconciliation error for device {:?}: {:?}",
        device.metadata.name, error
    );
    Action::await_change()
}
