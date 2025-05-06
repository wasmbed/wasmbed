# wasmbed

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

## Contributing

To ensure code quality and maintainability, please follow these guidelines when
contributing to the project.

### Code Requirements

* Editor Configuration: Your editor should respect the settings defined in the
  `.editorconfig` file to maintain consistent formatting across the codebase.
* Unit Testing: All code contributions must include appropriate unit tests
  that verify functionality.
* Test Verification: Before committing changes, ensure all tests pass by
  running `cargo test` and `cargo test -- --ignored`.
* Code Formatting: All code must be formatted using `rustfmt`. Run
  `cargo fmt` to apply formatting.
* Code Linting: Code must pass Clippy's linting checks. Run `cargo clippy`
  to verify.

Alternatively, you can run `nix flake check` to perform all the above checks at
once.

### Commit Messages

This project follows the [Conventional Commits][conventional-commits]
format. Each commit message should be structured as follows:

```
<type>(<optional scope>): <description>

[optional body]
```

Types include:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code refactoring without changing functionality
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

Example:

```
feat(protocol): Add new message type for device registration
```

Please split complex changes into multiple smaller commits when possible. Each
commit should be self-contained and leave the codebase in a buildable,
test-passing state.

[conventional-commits]: https://www.conventionalcommits.org/

### Committing

Direct commits to the `master` branch are not allowed. All contributions must
be made through branches and submitted via pull requests for review.

When creating a branch, use the following naming convention:
`<username>/<topic>`. E.g.:

- `alice/fix-connection-timeout`
- `bob/add-metrics-endpoint`

Once your branch is ready, open a pull request targeting the `master` branch. A
maintainer will review your changes and provide feedback or approval.

## Generating Diagrams from PlantUML Files

Please refer to [resources/diagrams/README.md](resources/diagrams/README.md).
