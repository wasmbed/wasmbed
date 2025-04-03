# wasmed

## Development Environment

This project uses Nix to manage the development environment, ensuring
consistency and reproducibility across systems while automatically handling
dependencies and tool configurations.

### Prerequisites

If you're using [NixOS][nixos], you don't need to install Nix separately, just
make sure flakes are enabled in your configuration.

For all other Linux distributions or macOS, we recommend using the
[Determinate Systems Nix installer][determinate-systems-nix] for a more robust
and user-friendly experience. See their website for installation instructions
and platform details.

[nixos]: https://nixos.org/
[determinate-systems-nix]: https://zero-to-nix.com/

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

- \<tool-package-name\>: The name of the tool's package (e.g., wasmbed-protocol-tool).
- \<args\>: Any arguments that the tool accepts (e.g. --hex).

## Generating Diagrams from PlantUML Files

Please refer to [resources/diagrams/README.md](resources/diagrams/README.md).
