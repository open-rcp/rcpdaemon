# rcpdaemon - RCP Daemon

This is the RCP daemon (rcpdaemon) that combines the functionality of the previously separate RCP server and service components into a single daemon process.

## Architecture

The daemon includes:
- Core process management functionality for lifecycle management
- Embedded server functionality for handling connections
- Optional API component (feature-gated)
- Unified configuration system
- Simplified deployment and operation

## Configuration

The service is configured through a single config file that includes both service and server settings:

```toml
# Service configuration
address = "127.0.0.1"
port = 8716

# TLS configuration for the service
[tls]
enabled = false
cert_path = "cert.pem"
key_path = "key.pem"

# Server configuration
[server]
address = "0.0.0.0"
port = 8717

# Server TLS configuration
[server.tls]
enabled = false
cert_path = "server-cert.pem"
key_path = "server-key.pem"

# Server authentication
[server.auth]
required = true
```

## Usage

### Running in Development Mode

```bash
# Run in foreground with a specific config
rcpdaemon -c config.toml -f

# Run with verbose output
rcpdaemon -c config.toml -f -v

# Run using cargo directly
cargo run -p rcpdaemon -- -c config.toml -f
```

### Running as a System Service

```bash
# Start the service (after installation)
# Linux:
sudo systemctl start rcpdaemon

# macOS:
sudo launchctl start com.devstroop.rcpdaemon

# Windows:
sc start rcpdaemon
```

### Stopping the Service

```bash
# Linux:
sudo systemctl stop rcpdaemon

# macOS:
sudo launchctl stop com.devstroop.rcpdaemon

# Windows:
sc stop rcpdaemon
```

### Command-line Options

```
USAGE:
    rcpdaemon [OPTIONS]

OPTIONS:
    -c, --config <FILE>     Path to config file [default: config.toml]
    -d, --daemon            Run as a background daemon
    -f, --foreground        Run in the foreground
    -h, --help              Print help information
    -v, --verbose           Verbose output
        --version           Print version information
```

## Benefits of Integration

1. **Simplified Deployment**: Single binary with integrated functionality
2. **Development Efficiency**: Easier to run, test, and debug
3. **Reduced Resource Usage**: Lower memory footprint, shared resources
4. **Better Error Handling**: No need to coordinate errors across process boundaries
5. **Unified Configuration**: Single configuration system for all components

## Installation

For detailed instructions on building and installing rcpdaemon as a system service (systemd, launchd, or Windows service), please refer to the [comprehensive installation guide](../docs/rcpdaemon-installation.md).

This directory includes ready-to-use service files:
- `systemd/rcpdaemon.service` - For Linux systems with systemd
- `launchd/com.devstroop.rcpdaemon.plist` - For macOS systems

For quick local installation instructions, see [INSTALL.md](INSTALL.md).
