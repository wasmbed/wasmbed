# Kubernetes Test Environment

To work with the Kubernetes test environment, we use [k3d], a lightweight
wrapper to run [k3s] in Docker. The project includes a custom configuration file
at `config.yaml` in this directory.

Before starting, make sure Docker is installed and correctly running in your
system.

[k3d]: https://k3d.io
[k3s]: https://k3s.io/

## Managing the Test Cluster

Due to a [limitation of k3d][k3d-relative-path-issue], you have to manually
adapt the config changing the absolute path to the resources/dev-certs directoy.

Create and start the cluster using the custom configuration:

```
k3d cluster create --config config.yaml
```

Ideally, you should run the create command once.

To stop the cluster without deleting it:

```
k3d cluster stop wasmbed
```

To start a previously created cluster:

```
k3d cluster start wasmbed
```

To completely remove the cluster:

```
k3d cluster delete wasmbed
```

[k3d-relarive-path-issue]: https://github.com/k3d-io/k3d/issues/1289

## Configuring kubectl

The custom k3d configuration does not automatically update your kubeconfig
file. After creating the cluster, you need to manually set the KUBECONFIG
environment variable in order to interact with it using kubectl:

```
export KUBECONFIG=$(k3d kubeconfig write wasmbed)
```

This command generates a kubeconfig file for the wasmbed cluster and sets your
KUBECONFIG environment variable to point to it. After running this command,
verify your connection to the cluster with:

```
kubectl cluster-info
```

Note that this setting only applies to your current terminal session. You'll
need to run the export command again if you open a new terminal window or tab.
