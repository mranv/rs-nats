## How to Use

This remote support system provides a way to execute commands remotely on client machines through NATS messaging system. It works on both Linux and Windows with the same codebase.

### Prerequisites

1. Have a NATS server running (by default it connects to `nats://localhost:4222`)
2. Install Rust and Cargo

### Building

```bash
# Clone the repository
git clone <your-repo>
cd rs-nats

# Build the project
cargo build --release
```

### Running the Server (Support Provider)

```bash
# On Linux/macOS
./target/release/rs-nats server

# On Windows
.\target\release\rs-nats.exe server
```

### Running the Client (on the machine to be supported)

```bash
# On Linux/macOS
./target/release/rs-nats client

# On Windows
.\target\release\rs-nats.exe client
```

### Custom Settings

You can customize the NATS server URL and subject prefix:

```bash
# Example with custom NATS server
./target/release/rs-nats --nats-url nats://my-server:4222 client

# Example with custom subject prefix for isolation
./target/release/rs-nats --subject-prefix my-company-support client
```

## Features

1. **Cross-Platform** - Works identically on both Windows and Linux
2. **Secure Command Execution** - Server can execute commands remotely on client machines
3. **System Information** - View system details of connected clients
4. **Interactive Console** - The server provides an interactive console for managing clients
5. **Heartbeat Monitoring** - Clients send regular heartbeats to maintain connection status

## Server Commands

- `list` - List all connected clients
- `execute <client_id> <command>` - Execute a command on a specific client
- `sysinfo <client_id>` - Get detailed system information
- `ping <client_id>` - Check if a client is responsive
- `exit` - Shut down the server
