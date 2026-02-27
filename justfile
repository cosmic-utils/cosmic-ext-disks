# Justfile for COSMIC Ext Storage development

# Default workflow: build, stop service, install policies, start service in background, run app
default:
    @just build
    @just service-stop
    @just install-policy
    @just service-start
    @just app
    @just service-stop

# Build all workspace crates
build:
    cargo build --workspace --locked

# Build all crates in release mode
release:
    cargo build --workspace --release --locked

# Canonical workspace verification flow
check:
    cargo clippy --workspace --all-targets
    cargo fmt --all
    cargo test --workspace

# Clean build artifacts
clean:
    cargo clean

# Watch for changes and rebuild
watch:
    cargo watch -x "build --workspace"

# Watch and run tests
watch-tests:
    cargo watch -x "test --workspace"

# Run harness integration workflow
harness:
    cargo build --workspace --locked
    cargo run -p storage-testing --bin harness -- run --runtime auto

# Run lab image create for the given spec (defaults to 2disk).
lab spec="2disk":
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p storage-testing --locked
    cargo run -p storage-testing --bin lab -- image create "{{spec}}"

# Monitor D-Bus signals
watch-dbus:
    @echo "Monitoring D-Bus signals from storage service..."
    dbus-monitor --system "type='signal',sender='org.cosmic.ext.Storage.Service'"

# Start the COSMIC Ext Storage UI
app: build
    #!/usr/bin/env bash
    echo "Starting COSMIC Ext Storage UI..."
    RUST_LOG=cosmic_ext_storage=debug,info,wgpu=warn,wgpu_core=warn,wgpu_hal=warn,naga=warn,iced_winit=warn,iced_wgpu=warn,i18n_embed=warn ./target/debug/cosmic-ext-storage

# Run service in foreground (interactive)
service: build
    #!/usr/bin/env bash
    echo "Starting storage service (requires root)..."
    sudo pkill -f cosmic-ext-storage-service || true
    sudo RUST_LOG=storage_service=debug,info ./target/debug/cosmic-ext-storage-service

# Start service in background
service-start: build
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting storage service in background (requires root)..."
    sudo -v
    sudo pkill -f cosmic-ext-storage-service || true
    sudo rm -f /tmp/cosmic-ext-storage-service.log
    sudo bash -c 'nohup env RUST_LOG=storage_service=info ./target/debug/cosmic-ext-storage-service > /tmp/cosmic-ext-storage-service.log 2>&1 &'
    sleep 2
    echo "Service started. Logs: /tmp/cosmic-ext-storage-service.log"
    if pgrep -f cosmic-ext-storage-service > /dev/null; then
        echo "✓ Service is running"
    else
        echo "✗ Service not running. Check logs: sudo cat /tmp/cosmic-ext-storage-service.log"
        exit 1
    fi

# Stop service
service-stop:
    #!/usr/bin/env bash
    echo "Stopping storage service..."
    sudo pkill -f cosmic-ext-storage-service || true
    echo "Service stopped"

# Service status
service-status:
    @echo "Storage service status:"
    @systemctl status cosmic-ext-storage-service.service || echo "Service not installed as systemd unit"
    @echo ""
    @echo "Process status:"
    @pgrep -af cosmic-ext-storage-service || echo "Service not running"

# Follow service logs
service-logs:
    @journalctl -u cosmic-ext-storage-service -f

# Introspect D-Bus service
service-introspect:
    @echo "Testing D-Bus interface..."
    @echo "Listing service..."
    busctl --system tree org.cosmic.ext.Storage.Service
    @echo ""
    @echo "Introspecting BTRFS interface..."
    busctl --system introspect org.cosmic.ext.Storage.Service /org/cosmic/ext/Storage/Service/btrfs

# Build and install system files & policies (requires root)
install: release install-policy
    @echo "Installing systemd service files..."
    sudo install -Dm644 resources/systemd/cosmic-ext-storage-service.service /usr/lib/systemd/system/
    sudo install -Dm644 resources/systemd/cosmic-ext-storage-service.socket /usr/lib/systemd/system/
    @echo "Installing service binary..."
    sudo install -Dm755 target/release/cosmic-ext-storage-service /usr/bin/
    @echo "Reloading systemd..."
    sudo systemctl daemon-reload
    @echo ""
    @echo "System files installed. You can now enable the service with:"
    @echo "  sudo systemctl enable --now cosmic-ext-storage-service.service"

# Install D-Bus and Polkit policies (requires root)
install-policy:
    @echo "Installing D-Bus policy..."
    sudo install -Dm644 resources/systemd/org.cosmic.ext.Storage.Service.conf /usr/share/dbus-1/system.d/
    @echo "Installing Polkit policy..."
    sudo install -Dm644 resources/systemd/org.cosmic.ext.storage.service.policy /usr/share/polkit-1/actions/
    @echo "Reloading D-Bus configuration..."
    sudo systemctl reload dbus || true
    @echo ""
    @echo "Policies installed."

# Install development dependencies using detected package manager
install-deps:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v apt-get >/dev/null 2>&1; then
        echo "Installing development dependencies (Debian/Ubuntu)..."
        sudo apt-get install -y \
            build-essential \
            pkg-config \
            libdbus-1-dev \
            libpolkit-gobject-1-dev \
            libbtrfs-dev \
            btrfs-progs \
            systemd \
            dbus
    elif command -v dnf >/dev/null 2>&1; then
        echo "Installing development dependencies (Fedora)..."
        sudo dnf install -y \
            gcc \
            pkg-config \
            dbus-devel \
            polkit-devel \
            btrfs-progs-devel \
            btrfs-progs \
            systemd \
            dbus
    elif command -v pacman >/dev/null 2>&1; then
        echo "Installing development dependencies (Arch)..."
        sudo pacman -S --needed \
            base-devel \
            pkg-config \
            dbus \
            polkit \
            btrfs-progs \
            systemd
    else
        echo "Unsupported package manager. Please install dependencies manually."
        exit 1
    fi
