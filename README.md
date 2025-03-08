# Remote Support Execution using NATS (rs-nats)

A cross-platform remote support tool built in Rust that enables secure command execution on remote machines over NATS messaging system. Works identically on both Windows and Linux.

## Features

- **Remote Command Execution**: Execute shell commands on client machines from the server
- **Cross-Platform**: Works on Windows and Linux with the same codebase
- **System Information**: View detailed system information about connected clients
- **Interactive Console**: Easy-to-use interactive server interface
- **Automatic Discovery**: Clients automatically register with the server
- **Secure Architecture**: Built on the reliable NATS messaging system

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) and Cargo (1.70.0+)
- A running [NATS Server](https://nats.io/) (local or remote)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/mranv/rs-nats.git
cd rs-nats

# Build the project
cargo build --release
```

The compiled binaries will be located at:
- `./target/release/rs-nats` (Linux/macOS)
- `.\target\release\rs-nats.exe` (Windows)

### Running NATS Server

If you don't already have a NATS server running, you can easily set one up:

#### Using Docker
```bash
docker run -p 4222:4222 -p 8222:8222 nats
```

#### Using the NATS binary
Download the appropriate binary from [NATS.io](https://nats.io/download/) and run it:
```bash
./nats-server
```

## Usage

### Server Mode (Support Provider)

Run the server on the machine that will be providing support:

```bash
# Linux/macOS
./target/release/rs-nats server

# Windows
.\target\release\rs-nats.exe server
```

### Client Mode (Support Recipient)

Run the client on the machine that needs support:

```bash
# Linux/macOS
./target/release/rs-nats client

# Windows
.\target\release\rs-nats.exe client
```

### Command Line Options

```
USAGE:
    rs-nats [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -n, --nats-url <URL>             NATS server URL [default: nats://localhost:4222]
    -s, --subject-prefix <PREFIX>    Subject prefix for NATS messages [default: rs-support]
    -h, --help                       Print help information
    -V, --version                    Print version information

SUBCOMMANDS:
    server    Run in server mode (support provider)
    client    Run in client mode (support recipient)
    help      Print this message or the help of the given subcommand(s)
```

#### Examples

Connect to a custom NATS server:
```bash
./target/release/rs-nats --nats-url nats://my-nats-server:4222 server
```

Use a custom subject prefix (for separation in shared NATS servers):
```bash
./target/release/rs-nats --subject-prefix mycompany-support client
```

Specify a custom client ID:
```bash
./target/release/rs-nats client --client-id workstation-5
```

## Server Commands

Once the server is running, you can use the following interactive commands:

| Command | Description |
|---------|-------------|
| `list` | List all connected clients with their details |
| `execute <client_id> <command>` | Execute a command on a specific client |
| `sysinfo <client_id>` | Get detailed system information from a client |
| `ping <client_id>` | Check if a client is responsive |
| `exit` | Shut down the server |

### Example Server Session

```
Available commands:
  list                - List connected clients
  execute <id> <cmd>  - Execute command on client
  sysinfo <id>        - Get system info from client
  ping <id>           - Ping client
  exit                - Exit server

list
Connected clients:
  john-laptop - John-Laptop (john / Linux)
  sarah-pc - Sarah-PC (sarah / Windows)

execute john-laptop ls -la
Executing command on john-laptop: ls -la

Client: john-laptop
Command result: Success
Output:
total 32
drwxr-xr-x  5 john john 4096 Mar 9 10:15 .
drwxr-xr-x 20 john john 4096 Mar 9 09:45 ..
drwxr-xr-x  3 john john 4096 Mar 9 10:15 target
-rw-r--r--  1 john john  118 Mar 9 09:45 Cargo.toml
-rw-r--r--  1 john john  142 Mar 9 09:45 Cargo.lock
drwxr-xr-x  2 john john 4096 Mar 9 09:45 src
```

## Security Considerations

- This tool allows remote command execution, which has inherent security risks
- Use only in trusted environments or secure networks
- Consider adding authentication mechanisms for production use
- Keep NATS server secure by using TLS and proper authentication

## Project Structure

```
rs-nats/
├── Cargo.toml           # Project configuration
├── Cargo.lock           # Dependency lock file
└── src/
    ├── main.rs          # CLI entry point
    ├── lib.rs           # Shared library components
    ├── client.rs        # Client implementation
    └── server.rs        # Server implementation
```

## Troubleshooting

### Connection Issues
- Ensure NATS server is running and accessible
- Check firewall settings to allow port 4222
- Verify correct NATS URL is provided

### Command Execution Problems
- Ensure client has appropriate permissions to run commands
- Check for platform-specific command syntax differences
- Verify client is connected with `list` command
