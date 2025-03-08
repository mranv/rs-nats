# Remote Support System

This project provides a remote support system using NATS messaging for reliable communication between a server (support provider) and multiple clients (machines requiring support).

## Features

- **Reliable Communication:** Utilizes NATS messaging for robust and efficient communication.
- **Cross-Platform Command Execution:** Executes commands on both Windows and Linux systems with appropriate shell selection.
- **System Information Gathering:** Collects detailed system information compatible with Windows and Linux.
- **Client Auto-Registration:** Automatically registers clients with hostname and username detection.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (for building the project)
- [Docker](https://docs.docker.com/get-docker/) (for running a NATS server)

## Building the Project

To build the project in release mode, run:

```bash
cargo build --release
```

## Setting Up the NATS Server

If you don't have a NATS server running, you can start one using Docker:

```bash
docker run -p 4222:4222 nats
```

## Running the Server (Support Provider Machine)

On the support provider's machine, start the server:

```bash
./target/release/rs-nats server
```

## Running the Client (Machines Needing Support)

On each client machine that requires support, run:

```bash
./target/release/rs-nats client
```

## Server Interactive Console Commands

Once the server is running, you can use the interactive console with the following commands:

- `list`  
  Show all connected clients.

- `execute <client_id> <command>`  
  Run a command on a specific client.

- `sysinfo <client_id>`  
  View detailed system information of a client.

- `ping <client_id>`  
  Check if a client is responsive.

- `exit`  
  Shut down the server.

## Implementation Details

- **NATS Messaging:** Ensures reliable communication between the server and clients.
- **Error Handling:** Provides detailed error handling for command execution.
- **Cross-Platform Compatibility:** Executes commands using the appropriate shell for Windows and Linux.
- **System Information:** Gathers comprehensive system information compatible with both Windows and Linux.
- **Client Auto-Registration:** Clients automatically register upon connection, with detection of hostname and username.

For any questions about implementation details or extending functionality, please reach out. 