# wasmed

## Project Structure

- Libraries
  - [virtual-kubelet-client][virtual-kubelet-client]: Client for the
    communication protocol used by the Virtual Kubelet
- Tooling
  - [virtual-kubelet-cbor-inspect][virtual-kubelet-cbor-inspect]: Tool for
    deserializing CBOR data and inspecting the encoded Virtual Kubelet
    communication protocol.

[virtual-kubelet-client]: ./crates/virtual-kubelet-client
[virtual-kubelet-cbor-inspect]: ./tools/virtual-kubelet-cbor-inspect

## Development Environment

This project uses Nix to manage the development environment, ensuring
consistency and reproducibility across systems while automatically handling
dependencies and tool configurations.

### Prerequisites

- Install the [Nix package manager][nix-download] on your system.

[nix-download]: https://nixos.org/download/

### Setting Up

To enter the development environment, run the following command:

```
nix develop
```

This will start a new shell with the development environment, automatically
installing all necessary dependencies.

## Testing

To run the tests, run the following command:

```
cargo test
```

## Tooling

To run a specific tool in the project, you can use the `cargo run` command with
the `-p` flag followed by the tool's package name. This will allow you to execute
the tool with any additional arguments you need.

```
cargo run -p <tool-package-name> -- <args>
```

- \<tool-package-name\>: The name of the tool's package (e.g.,
virtual-kubelet-cbor-inspect).
- \<args\>: Any arguments that the tool accepts (e.g. --hex).
