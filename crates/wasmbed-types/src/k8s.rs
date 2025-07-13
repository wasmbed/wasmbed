// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use alloc::borrow::ToOwned;
use alloc::string::ToString;

use k8s_openapi::api::core::v1::TypedObjectReference;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct GatewayReference(
    pub k8s_openapi::api::core::v1::TypedObjectReference,
);

impl GatewayReference {
    pub fn new(namespace: &str, name: &str) -> Self {
        Self(TypedObjectReference {
            api_group: None,
            kind: "Pod".to_string(),
            name: name.to_owned(),
            namespace: Some(namespace.to_owned()),
        })
    }
}
