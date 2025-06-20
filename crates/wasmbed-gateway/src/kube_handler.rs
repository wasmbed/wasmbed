use wasmbed_k8s_resource::Device;
use kube::{Client};
use kube::api::{Api, DeleteParams, Patch, PostParams, PatchParams};
use kube::Error;
use either::Either::{Left, Right};

pub struct KubeClient(Client);

impl KubeClient {
    pub async fn new() -> Result<Self, Error> {
        let client = Client::try_default().await?;
        Ok(KubeClient(client))
    }

    pub async fn add_device(
        &self,
        device: Device,
        namespace: String,
    ) -> Result<(), Error> {
        let device_crd: Api<Device> =
            Api::namespaced(self.0.clone(), &namespace);
        let _ = device_crd.create(&PostParams::default(), &device).await?;

        Ok(())
    }

    pub async fn delete_device(
        &self,
        device_pub_key: &str,
        namespace: String,
    ) -> Result<(), Error> {
        let device_crd: Api<Device> =
            Api::namespaced(self.0.clone(), &namespace);
        match device_crd
            .delete(device_pub_key, &DeleteParams::default())
            .await
        {
            Ok(response) => match response {
                Left(deleted) => {
                    println!("Deleted: {:?}", deleted.metadata.name);
                    Ok(())
                },
                Right(status) => {
                    println!("Delete started: status = {:?}", status.status);
                    Ok(())
                },
            },
            Err(e) => Err(e),
        }
    }

    pub async fn update_device(
        &self,
        device_pub_key: &str,
        device: Device,
        namespace: &str,
    ) -> Result<(), Error> {
        let device_crd: Api<Device> =
            Api::namespaced(self.0.clone(), namespace);
        match device_crd
            .patch(
                device_pub_key,
                &PatchParams::default().force(),
                &Patch::Apply(device),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
