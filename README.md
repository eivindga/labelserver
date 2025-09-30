# Label Server

A REST API server for printing labels on Dymo labelwriters via CUPS.

## Prerequisites

- Rust (installed automatically if not present)
- CUPS system with Dymo labelwriter configured
- Dymo labelwriter connected via USB

## Setup

1. Ensure your Dymo labelwriter is connected and configured in CUPS:
   ```bash
   lpstat -p  # List available printers
   ```

2. Build and run the server:
   ```bash
   cargo run
   ```

The server will start on `http://0.0.0.0:3000`

## Web Interface

The server includes a web interface accessible at `http://localhost:3000` that provides:
- 4 text input fields for label content
- Label format selection dropdown (default: 99012)
- Print button with status feedback
- Clean, responsive design for easy label printing

## API Endpoints

### Health Check
```
GET /
GET /health
```
Returns server status and version.

### List Printers
```
GET /printers
```
Returns a list of available printers.

### Print Label
```
POST /print
Content-Type: application/json

{
  "line1": "John Doe",                     // Required - First line of text
  "line2": "123 Main Street",              // Optional - Second line of text
  "line3": "Anytown, NY 12345",            // Optional - Third line of text
  "line4": "USA",                          // Optional - Fourth line of text
  "printer_name": "DYMO_LabelWriter_450",  // Optional
  "label_size": "30252"                    // Optional - Dymo label size code
}
```

- `line1` is required and cannot be empty
- `line2`, `line3`, and `line4` are optional
- Empty lines are automatically filtered out when printing
- If `printer_name` is not specified, the server will automatically find the first available Dymo printer
- If `label_size` is not specified, it defaults to "30252" (standard Dymo address label)

#### Common Dymo Label Sizes
- `30252` - Address labels (1-1/8" x 3-1/2")
- `30256` - Shipping labels (2-5/16" x 4")
- `30321` - Large address labels (1-4/10" x 3-1/2")
- `30330` - Return address labels (3/4" x 2")
- `99012` - Large address labels (3-1/2" x 1-1/8")

## Example Usage

```bash
# Check server health
curl http://localhost:3000/health

# List available printers
curl http://localhost:3000/printers

# Print a simple one-line label
curl -X POST http://localhost:3000/print \
  -H "Content-Type: application/json" \
  -d '{"line1": "Hello, World!"}'

# Print a complete address label
curl -X POST http://localhost:3000/print \
  -H "Content-Type: application/json" \
  -d '{
    "line1": "John Doe",
    "line2": "123 Main Street",
    "line3": "Anytown, NY 12345",
    "line4": "USA"
  }'

# Print to specific printer
curl -X POST http://localhost:3000/print \
  -H "Content-Type: application/json" \
  -d '{
    "line1": "Jane Smith",
    "line2": "456 Oak Avenue",
    "printer_name": "DYMO_LabelWriter_450"
  }'

# Print with custom label size (shipping label)
curl -X POST http://localhost:3000/print \
  -H "Content-Type: application/json" \
  -d '{
    "line1": "URGENT DELIVERY",
    "line2": "ABC Company",
    "line3": "789 Business Blvd",
    "label_size": "30256"
  }'

# Print return address with small label size
curl -X POST http://localhost:3000/print \
  -H "Content-Type: application/json" \
  -d '{
    "line1": "From: My Company",
    "line2": "PO Box 123",
    "printer_name": "DYMO_LabelWriter_450",
    "label_size": "30330"
  }'
```

## Configuration

The server uses default Dymo label size (30252) when no `label_size` is specified in the request. You can override this per-request by including a `label_size` parameter in your print requests.

## Running as a Service (Raspberry Pi / Debian)

For production deployment on a Raspberry Pi or Debian-based system, you can set up the label server as a systemd service to automatically start on boot and restart on crashes.

### Prerequisites for Raspberry Pi

1. **Install CUPS and Dymo drivers:**
   ```bash
   sudo apt update
   sudo apt install cups cups-client printer-driver-dymo
   ```

2. **Configure CUPS to accept connections:**
   ```bash
   sudo usermod -a -G lpadmin pi
   sudo systemctl enable cups
   sudo systemctl start cups
   ```

3. **Connect and configure your Dymo printer via CUPS web interface:**
   ```bash
   # Access CUPS web interface at http://your-pi-ip:631
   # Or configure via command line:
   sudo lpadmin -p DYMO_LabelWriter_450 -E -v usb://DYMO/LabelWriter%20450 -m dymo_lw450.ppd
   ```

   > **Detailed Setup Guide**: For a comprehensive step-by-step guide on setting up a Raspberry Pi as a print server for Dymo label printers, including CUPS configuration and troubleshooting, see: [Configure a Raspberry Pi as a Print Server for Dymo Label Printers](https://johnathan.org/configure-a-raspberry-pi-as-a-print-server-for-dymo-label-printers/)

### Build and Install for Production

#### For x86_64 Systems (Standard)

1. **Build the optimized release binary:**
   ```bash
   cd /path/to/labelserver
   cargo build --release
   ```

#### For Raspberry Pi 2/3 (32-bit ARMv7)

If you're building on a different architecture (e.g., x86_64) to deploy on Raspberry Pi 2 with 32-bit Debian Bookworm:

1. **Install cross-compilation tools:**
   ```bash
   # Install Rust ARM target (musl for better compatibility)
   rustup target add arm-unknown-linux-musleabihf

   # Install ARM cross-compiler (on Ubuntu/Debian)
   sudo apt install gcc-arm-linux-gnueabihf
   ```

2. **Create cargo cross-compilation config:**
   ```bash
   mkdir -p .cargo
   echo '[target.arm-unknown-linux-musleabihf]
   linker = "arm-linux-gnueabihf-gcc"' > .cargo/config.toml
   ```

3. **Build for ARM:**
   ```bash
   cargo build --release --target arm-unknown-linux-musleabihf
   ```

   The ARM binary will be located at: `target/arm-unknown-linux-musleabihf/release/labelserver`

   > **Note**: This uses musl libc for static linking, making it compatible with older glibc versions on Raspberry Pi systems.

#### Install the Binary and Static Files

**Create installation directory:**
```bash
sudo mkdir -p /opt/labelserver/static
```

**For x86_64 deployment:**
```bash
sudo cp target/release/labelserver /opt/labelserver/
sudo cp static/index.html /opt/labelserver/static/
sudo chmod +x /opt/labelserver/labelserver
```

**For Raspberry Pi 2 deployment:**
```bash
sudo cp target/arm-unknown-linux-musleabihf/release/labelserver /opt/labelserver/
sudo cp static/index.html /opt/labelserver/static/
sudo chmod +x /opt/labelserver/labelserver
```

**Set ownership and create symlink:**
```bash
# Set ownership
sudo chown -R labelserver:labelserver /opt/labelserver

# Create symlink for easy access (optional)
sudo ln -sf /opt/labelserver/labelserver /usr/local/bin/labelserver
```

#### Create a Dedicated User (Optional but Recommended)

```bash
sudo useradd --system --shell /bin/false --home /opt/labelserver labelserver
```

### Create Systemd Service

1. **Create the service file:**
   ```bash
   sudo nano /etc/systemd/system/labelserver.service
   ```

2. **Add the following configuration:**
   ```ini
   [Unit]
   Description=Label Server - Dymo Label Printer REST API
   Documentation=https://github.com/your-repo/labelserver
   After=network-online.target cups.service
   Wants=network-online.target
   Requires=cups.service

   [Service]
   Type=simple
   User=labelserver
   Group=lp
   ExecStart=/opt/labelserver/labelserver
   WorkingDirectory=/opt/labelserver
   Environment=RUST_LOG=info

   # Restart policy
   Restart=always
   RestartSec=5

   # Security settings
   NoNewPrivileges=yes
   PrivateTmp=yes
   ProtectSystem=strict
   ProtectHome=yes
   ReadWritePaths=/opt/labelserver

   # Resource limits
   LimitNOFILE=1024

   [Install]
   WantedBy=multi-user.target
   ```

### Service Management

1. **Reload systemd and enable the service:**
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable labelserver.service
   ```

2. **Start the service:**
   ```bash
   sudo systemctl start labelserver.service
   ```

3. **Check service status:**
   ```bash
   sudo systemctl status labelserver.service
   ```

4. **View service logs:**
   ```bash
   # View recent logs
   sudo journalctl -u labelserver.service -f

   # View logs since boot
   sudo journalctl -u labelserver.service --since today

   # View last 50 log entries
   sudo journalctl -u labelserver.service -n 50
   ```

5. **Stop or restart the service:**
   ```bash
   sudo systemctl stop labelserver.service
   sudo systemctl restart labelserver.service
   ```

6. **Disable auto-start (if needed):**
   ```bash
   sudo systemctl disable labelserver.service
   ```

### Service Troubleshooting

- **Check if the service is running:**
  ```bash
  sudo systemctl is-active labelserver.service
  ```

- **Check if the service is enabled for boot:**
  ```bash
  sudo systemctl is-enabled labelserver.service
  ```

- **Test the binary manually:**
  ```bash
  /opt/labelserver/labelserver
  ```

- **Check CUPS dependencies:**
  ```bash
  sudo systemctl status cups.service
  lpstat -p  # Should show your Dymo printer
  ```

- **Verify network connectivity:**
  ```bash
  curl http://localhost:3000/health
  ```

- **Check service permissions:**
  ```bash
  # Ensure the labelserver user can access CUPS
  groups labelserver  # Should include 'lp' group
  ```

### Updating the Service

When updating the label server:

1. **Stop the service:**
   ```bash
   sudo systemctl stop labelserver.service
   ```

2. **Update the binary and static files:**

   **For x86_64:**
   ```bash
   cargo build --release
   sudo cp target/release/labelserver /opt/labelserver/
   sudo cp static/index.html /opt/labelserver/static/
   ```

   **For Raspberry Pi 2:**
   ```bash
   cargo build --release --target arm-unknown-linux-musleabihf
   sudo cp target/arm-unknown-linux-musleabihf/release/labelserver /opt/labelserver/
   sudo cp static/index.html /opt/labelserver/static/
   ```

3. **Start the service:**
   ```bash
   sudo systemctl start labelserver.service
   ```

## Troubleshooting

- Ensure CUPS is running: `systemctl status cups`
- Check printer status: `lpstat -p`
- Verify printer queues: `lpq -P your_printer_name`