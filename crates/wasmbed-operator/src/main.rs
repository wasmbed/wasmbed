mod device;

use crate::device::{Device, reconcile_device, on_reconcile_device_error};
use futures::StreamExt;
use kube::api::Api;
use kube::client::Client;
use kube::runtime::Controller;
use std::sync::Arc;
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::try_default().await?;
    let client_arc = Arc::new(client.clone());

    let devices: Api<Device> = Api::all(client.clone());

    let device_controller = Controller::new(devices, Default::default())
        .run(reconcile_device, on_reconcile_device_error, Arc::clone(&client_arc))
        .for_each(|_| futures::future::ready(()));

    device_controller.await;

    Ok(())
}
