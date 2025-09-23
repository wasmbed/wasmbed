# TLS Client (no_std, Ed25519)

This client uses [embedded-tls](https://github.com/fratrung/embedded-tls) to establish secure **TLS 1.3** connections in `no_std` embedded environments.  

It provides a `Client` struct that wraps:
- `TlsContext` and `TlsConnection` from `embedded-tls`  
- Certificate handling with Ed25519 (CA, client certificate, private key in DER format)  
- Async TCP sockets from [`embassy-net`](https://github.com/embassy-rs/embassy)  

The client supports connection setup, certificate verification, secure message exchange, and session management, all without relying on `std`.

---

## 📦 Dependencies

- [embedded-tls (Ed25519 fork)](https://github.com/fratrung/embedded-tls)  
- [embassy-net](https://github.com/embassy-rs/embassy)  
- [minicbor](https://docs.rs/minicbor)  
- [ed25519-dalek](https://docs.rs/ed25519-dalek)  
- [rand_core](https://docs.rs/rand_core)  

---

## 📖 License

- SPDX-License-Identifier: AGPL-3.0  
- Copyright © 2025 Wasmbed contributors  