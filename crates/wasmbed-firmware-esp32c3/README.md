# ESP32-C3 Heartbeat Firmware Demo

This crate contains a demo firmware for the **ESP32-C3** microcontroller.  
The firmware establishes a secure **mTLS connection** with a deployed gateway and periodically sends heartbeat messages.  

## How It Works

1. **Certificates**  
   - During the build process, the firmware embeds TLS client certificates.  
   - These certificates are expected to be located in:  
     ```
     resources/dev-certs
     ```
   - The gateway you connect to must be deployed with the **same certificates** in order for mutual TLS (mTLS) authentication to succeed.

2. **Gateway**  
   - The ESP32-C3 assumes there is a reachable gateway service running in a cluster.  
   - The gateway must be configured with the same certificates to accept the connection.  

3. **Firmware behavior**  
   - On startup, the ESP32-C3 initializes WiFi.  
   - It connects to the gateway using **mutual TLS**.  
   - Once connected, it sends **periodic heartbeat messages** to the gateway.

---

## Requirements

- **ESP32-C3 board**  
- [`espflash`](https://github.com/esp-rs/espflash) for flashing the firmware onto the device  
- Nix environment (`nix develop`) to ensure the correct toolchain  

---

## Running the Demo

To make testing easier, a helper script [`heartbeat_demo.sh`](./heartbeat_demo.sh) is provided.  

This script:  
1. Checks if `espflash` is installed (and installs it if missing)  
2. Navigates automatically into `crates/wasmbed-firmware-esp32c3` (from any starting directory)  
3. Prompts you for WiFi credentials and gateway connection details  
4. Builds the firmware with those environment variables (`cargo build --release`)  
5. Runs the firmware locally and streams the logs (`cargo run --release`)  

---

## Usage

First, enter the **Nix development environment**:

```bash
nix develop
```
Make script executable:

```bash
chmod +x heartbeat_demo.sh
