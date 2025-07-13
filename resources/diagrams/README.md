# Diagrams

This directory contains diagrams defined using [PlantUML](https://plantuml.com/).
Diagrams are written in `.puml` files and can be rendered to SVG format using `make`.

## Diagram Conventions

| Symbol | Description                |
|--------|----------------------------|
| `->`   | Synchronous call           |
| `->>`  | Asynchronous message       |
| `-->`  | Response or return message |

Custom resources defined through CRDs are wave-underlined using a pair of `~~`
(e.g. `~~Resource~~`).

## Generating Diagrams with Nix

To generate all the diagrams from their `.puml` files, run from the root
directory of this repository:

```sh
nix build '.#wasmbed-diagrams'
```

You will find the generated `.svg` files in the `result` directory.

## Generating Diagrams without Nix

If necessary, you can override the `plantuml` binary used by `make` by setting
the `PLANTUML` variable:

```sh
make PLANTUML=/path/to/plantuml svg
```

### Generating Diagrams

To generate an individual SVG diagram from its `.puml` file:

```sh
make <diagram>.svg
```

For example:

```sh
make application-deployment-workflow.svg
```

This will read `application-deployment-workflow.puml` and produce
`application-deployment-workflow.svg`.

To generate all available SVG diagrams:

```sh
make svg
```

### Cleaning Generated Files

To remove all generated `.svg` files:

```sh
make clean
```

## License

All diagrams in this directory are licensed under the [Creative Commons
Attribution 4.0 International License (CC BY 4.0)][cc-by-4.0].

[cc-by-4.0]: https://creativecommons.org/licenses/by/4.0
