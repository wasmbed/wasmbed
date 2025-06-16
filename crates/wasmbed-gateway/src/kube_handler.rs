use wasmbed::
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
    pub async fn add_device() -> Result<()> {


    }
}

