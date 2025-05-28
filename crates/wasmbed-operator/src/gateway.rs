use std::sync::Arc;
use kube::{CustomResource, Error};
use kube::runtime::controller::Action;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    kind = "Gateway",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySpec {
    gateway: String,
    pairing: bool,
}

pub async fn reconcile_device(
    _device: Arc<Gateway>,
    _ctx: Arc<()>,
) -> Result<Action, Error> {
    Ok(Action::await_change())
}

pub fn on_reconcile_device_error(
    device: Arc<Gateway>,
    error: &kube::Error,
    _ctx: Arc<()>,
) -> Action {
    println!(
        "Reconciliation error for device {:?}: {:?}",
        device.metadata.name, error
    );
    Action::await_change()
}
