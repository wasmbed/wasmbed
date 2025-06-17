use wasmbed::wasmbed_k8s_resource::Device;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{
        Api, DeleteParams, ListParams, Patch, PatchParams, PostParams,
        ResourceExt,
    },
    core::crd::CustomResourceExt,
    Client, CustomResource,
};


pub struct KubeClient{Client};

impl KubeClient {
    pub async fn new() -> Result<Self> {
        Client::try_default().await?
    }

    pub async fn add_device(&self, device: Device, namespace: String) -> Result<()> {
        let device_crd: Api<Device> = Api::namespaced(self.0, namespace);
        device_crd.create(&PostParams::default(), &device).await?
    }

    pub async fn delete_device(&self, device_pub_key: String, namespace: String) -> Result<()> {
        let device_crd: Api<Device> = Api::namespaced(self.0, namespace);
        match device_crd.delete(device_pub_key,&DeleteParams::default()).await {
            Ok(response) => match response {
                kube::api::Either::Left(deleted) => {
                    println!("Deleted: {:?}", deleted.metadata.name);
                }
                kube::api::Either::Right(status) => {
                    println!("Delete started: status = {:?}", status.status);
                }
            },
            Err(e) => {
                println!("Failed to delete Foo: {e}");
            }
        }
    }

    pub async fn update_device(&self, device_pub_key: String,device: Device, namespace: String) -> Result<()> {
        let device_crd: Api<Device> = Api::namespaced(self.0, namespace);
        match device_crd.patch(device_pub_key,&PathcParamas::default().force(), &Patch::Apply(device)).await {
            Ok(_) => Ok(())
            Err(e) => println!("Failed to update Device: {e}");
        }
    }
}

