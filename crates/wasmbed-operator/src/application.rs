use std::sync::Arc;
use kube::{CustomResource, Error};
use kube::runtime::controller::Action;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use wasmbed_types::DeviceId;

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Eq, PartialEq)]
enum ApplicationStatus {
    Ready,
    Pending,
    Error(String),
}

impl Default for ApplicationStatus {
    fn default() -> Self {
        ApplicationStatus::Pending
    }
}

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
    kind = "Application",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSpec {
    id: DeviceId,
    device_id: DeviceId,
    status: ApplicationStatus,
}

pub async fn reconcile_device(
    _device: Arc<Application>,
    _ctx: Arc<()>,
) -> Result<Action, Error> {
    Ok(Action::await_change())
}

pub fn on_reconcile_device_error(
    device: Arc<Application>,
    error: &kube::Error,
    _ctx: Arc<()>,
) -> Action {
    println!(
        "Reconciliation error for device {:?}: {:?}",
        device.metadata.name, error
    );
    Action::await_change()
}
