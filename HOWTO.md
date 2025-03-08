# Remote Support NATS Client/Server

This code implements a robust remote support tool that allows you to execute commands on client machines from a central server using NATS messaging.

## How to Build and Run

1. **First, ensure a NATS server is running**:
   ```bash
   # Using Docker
   docker run -p 4222:4222 nats
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run the server** (on your support machine):
   ```bash
   ./target/release/rs-nats server
   ```

4. **Run the client** (on machines to be supported):
   ```bash
   ./target/release/rs-nats client
   ```

## Server Commands

Once the server is running, you can use the following commands:

- `list` - Shows all connected clients
- `execute <client_id> <command>` - Run a shell command on a client
- `sysinfo <client_id>` - Get system information from a client
- `ping <client_id>` - Check if a client is responsive
- `exit` - Shut down the server

## Expected Behavior

1. The client will keep trying to connect until a server is available
2. Once connected, you can run commands remotely
3. You'll see command output in the server console
4. Even if the server restarts, clients will reconnect automatically

## Troubleshooting

If you don't see command output on the server:
- Check the logs for both client and server
- Verify the NATS server is running and accessible
- Make sure firewalls allow connections on port 4222

The extra logging we've added should help diagnose any connectivity issues.