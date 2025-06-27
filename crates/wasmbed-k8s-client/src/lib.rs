use kube::Error;
use wasmbed_types::PublicKey;
use wasmbed_k8s_resource::Device;
use kube::{Client, CustomResource};
use kube::Config;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{
    Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt,
};

pub struct K8sClient {
    pub client: Client,
}

impl K8sClient {
    // Create a new K8s Client from Config Struct
    pub fn new(config: Config) -> Result<Self, Error> {
        match Client::try_from(config) {
            Ok(client) => Ok(Self { client }),
            Err(e) => Err(e),
        }
    }

    // Maybe is not Necessary
    pub fn delete_device(&self) -> Result<(), Error> {
        todo!()
    }

    // Maybe is not Necessary
    pub async fn create_device(&self, device: Device) -> Result<Device, Error> {
        let crds: Api<CustomResourceDefinition> = Api::all(self.client.clone());
        let foocrd = Foo::crd();
        let pp = PostParams::default();
        devices.create(&pp, &device).await?;
    }

    // Get all the Devices
    pub async fn get_devices(
        &self,
        namespace: &str,
    ) -> Result<ObjectList<Device>, Error> {
        let devices: Api<Device> =
            Api::namespaced(self.client.clone(), namespace);
        let lp = ListParams::default();

        devices.list(&lp).await?
    }

    // Retrieve a Device by public key
    pub async fn get_device(
        &self,
        pub_key: PublicKey,
        namespace: &str,
    ) -> Result<Device, Error> {
        let devices: Api<Device> =
            Api::namespaced(self.client.clone(), namespace);

        devices.get(pub_key).await?
    }

    // Verify if if device is Registerd
    pub async fn verify_device(
        &self,
        pub_key: PublicKey,
        namespace: &str,
    ) -> Result<Option<PublicKey>, Error> {
        match self.get_device(pub_key, namespace).await {
            Ok(device) => Ok(Some(device.spec.public_key)),
            Err(Error::Api(ae)) if ae.code == 404 => Ok(None),
            Err(e) => Err(e),
        }
    }
}
