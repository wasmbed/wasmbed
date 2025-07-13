# Development Certificates

This directory contains development-only certificates used for local testing of
the Wasmbed gateway servers and clients. All certificates are generated using
the `wasmbed-cert-tool` command line tool.

You can invoke the tool like this:

```
cargo run -p wasmbed-cert-tool -- <arguments>
```

or with:

```
cd crates/wasmbed-cert-tool
cargo run -- <arguments>
```

or through Nix with:

```
nix run '.#wasmbed-cert-tool' <arguments>
```

## Generating CA Certificates

### Server Certificate Authority

```
cargo run -p wasmbed-cert-tool --                       \
  generate-ca server                                    \
  --common-name "Wasmbed Gateway Server Development CA" \
  --out-key resources/dev-certs/server-ca.key           \
  --out-cert resources/dev-certs/server-ca.der
```

### Client Certificate Authority

```
cargo run -p wasmbed-cert-tool --                       \
  generate-ca client                                    \
  --common-name "Wasmbed Gateway Client Development CA" \
  --out-key resources/dev-certs/client-ca.key           \
  --out-cert resources/dev-certs/client-ca.der
```

## Issuing Leaf Certificates

### Server Certificate

```
cargo run -p wasmbed-cert-tool --             \
  issue-cert server                           \
  --ca-key resources/dev-certs/server-ca.key  \
  --ca-cert resources/dev-certs/server-ca.der \
  --common-name "Wasmbed Gateway Server 0"    \
  --out-key resources/dev-certs/server-0.key  \
  --out-cert resources/dev-certs/server-0.der
```

### Client certificate

```
cargo run -p wasmbed-cert-tool --             \
  issue-cert client                           \
  --ca-key resources/dev-certs/client-ca.key  \
  --ca-cert resources/dev-certs/client-ca.der \
  --common-name "Wasmbed Gateway Client 0"    \
  --out-key resources/dev-certs/client-0.key  \
  --out-cert resources/dev-certs/client-0.der
```

## License

These certificates are intended for development and testing purposes only. They
are not subject to copyright and may be freely used, modified, or regenerated.
