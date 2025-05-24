# RCP Daemon Installation Guide

## System Service Files

This directory contains installation files for running rcpdaemon as a system service on different platforms:

- `systemd/rcpdaemon.service`: For Linux systems with systemd
- `launchd/com.devstroop.rcpdaemon.plist`: For macOS systems

## Installation Instructions

### Linux (systemd)

1. Copy the service file to systemd:
   ```
   sudo cp systemd/rcpdaemon.service /etc/systemd/system/
   ```

2. Reload systemd:
   ```
   sudo systemctl daemon-reload
   ```

3. Enable and start the service:
   ```
   sudo systemctl enable rcpdaemon
   sudo systemctl start rcpdaemon
   ```

### macOS (launchd)

1. Copy the plist file to LaunchDaemons:
   ```
   sudo cp launchd/com.devstroop.rcpdaemon.plist /Library/LaunchDaemons/
   ```

2. Load the service:
   ```
   sudo launchctl load -w /Library/LaunchDaemons/com.devstroop.rcpdaemon.plist
   ```

### Windows

For Windows systems, use the Windows Service Manager to install rcpdaemon as a service:

```
rcpdaemon.exe --install-service
```

Or use the service installation script:

```
scripts\windows\install_service.bat
```
