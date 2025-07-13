# Cloud Deployment Guide

This guide describes how to deploy the Wasmbed Gateway using k3d. With minimal
modifications, the instructions can also be adapted for a standard Kubernetes
cluster.

If you're using `k3d`, ensure you have followed the setup steps described in the
[k3d guide][k3d-readme].

[k3d-readme]: ../k3d/README.md

## Build and Import the Gateway Docker Image

From the root of the project, build the Gateway container image using `nix`:

```bash
nix build '.#dockerImages.x86_64-linux.wasmbed-gateway'
```

Then, import the image into your local Docker registry:

```bash
docker load -i $(readlink result)
```

The output should confirm that the image has been loaded, for example:

```
Loaded image: wasmbed-gateway:mk1zndvpsjkmxkrcwx7r74wdc0jmljb8
```

Next, import the image into the running `k3d` cluster (replace the tag with the
one output by `docker load`):

```bash
k3d image import -c wasmbed wasmbed-gateway:mk1zndvpsjkmxkrcwx7r74wdc0jmljb8
```

To verify that the image has been successfully imported into the cluster:

```bash
docker exec -it k3d-wasmbed-server-0 crictl images
```

## Create the `wasmbed` Namespace

Apply the namespace definition:

```bash
kubectl apply -f resources/k8s/000-namespace.yaml
```

## Apply the CRD Configuration

First, install the Custom Resource Definition (CRD) for `Device`:

```bash
cargo run -p wasmbed-k8s-resource-tool crd device | kubectl -n wasmbed apply -f -
```

Then, create a Device resource in the cluster:

```bash
cargo run -p wasmbed-k8s-resource-tool manifest device \
  --name device-0                                      \
  --cert resources/dev-certs/client-0.der              \
| kubectl -n wasmbed apply -f -
```

## Deploy the Gateway

Before proceeding, ensure that the container image reference in the [Gateway
StatefulSet configuration][gateway-statefulset] matches the one built earlier.

Then apply the Kubernetes resources related to the Gateway:

```bash
kubectl apply -f resources/k8s/100-service-account-gateway.yaml
kubectl apply -f resources/k8s/101-cluster-role-gateway-device-access.yaml
kubectl apply -f resources/k8s/102-cluster-rolebinding-gateway.yaml
kubectl apply -f resources/k8s/110-service-gateway.yaml
kubectl apply -f resources/k8s/111-statefulset-gateway.yaml
```

[gateway-statefulset]: 111-gateway-statefulset.yaml

## Test the Gateway

To expose the Gateway locally, use `kubectl port-forward`:

```bash
kubectl -n wasmbed port-forward service/wasmbed-gateway-service 4423:4423
```

Then, run the Gateway test client:

```bash
cargo run -p wasmbed-gateway-test-client --      \
  --address 127.0.0.1:4423                       \
  --server-ca resources/dev-certs/server-ca.der  \
  --private-key resources/dev-certs/client-0.key \
  --certificate resources/dev-certs/client-0.der
```

## License

The configuration files in this directory are released under the [MIT No
Attribution License (MIT-0)][mit-0]. You may freely use, modify, and distribute
them without restriction.

[mit-0]: https://spdx.org/licenses/MIT-0.html
