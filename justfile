# Justfile for COSMIC Ext Storage development

# Default recipe - full development workflow
# Builds, installs policies, starts service in background, and launches the UI
default:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "============================================"
    echo "  COSMIC Ext Storage Development Environment"
    echo "============================================"
    echo ""

    # Step 1: Build workspace
    echo "► Step 1/5: Building workspace..."
    if ! cargo build --workspace; then
        echo "✗ Build failed. Fix compilation errors and try again."
        exit 1
    fi
    echo "✓ Build complete"
    echo ""

    # Step 2: Install development policies (requires sudo)
    echo "► Step 2/5: Installing D-Bus and Polkit policies..."
    if ! sudo -v; then
        echo "✗ Sudo authentication required for policy installation."
        echo "  Run 'sudo -v' first or ensure you have sudo privileges."
        exit 1
    fi
    echo "  Installing D-Bus policy..."
    if ! sudo install -Dm644 data/dbus-1/system.d/org.cosmic.ext.StorageService.conf /usr/share/dbus-1/system.d/; then
        echo "✗ Failed to install D-Bus policy"
        exit 1
    fi
    echo "  Installing Polkit policy..."
    if ! sudo install -Dm644 data/polkit-1/actions/org.cosmic.ext.storage-service.policy /usr/share/polkit-1/actions/; then
        echo "✗ Failed to install Polkit policy"
        exit 1
    fi
    echo "  Reloading D-Bus configuration..."
    sudo systemctl reload dbus || true
    echo "✓ Policies installed"
    echo ""

    # Step 3: Stop any existing service
    echo "► Step 3/5: Stopping any existing storage service..."
    sudo pkill -f cosmic-storage-service || true
    sleep 1
    echo "✓ Service stopped (if was running)"
    echo ""

    # Step 4: Start service in background
    echo "► Step 4/5: Starting storage service in background..."
    sudo rm -f /tmp/cosmic-storage-service.log
    sudo bash -c 'nohup env RUST_LOG=storage_service=info ./target/debug/cosmic-storage-service > /tmp/cosmic-storage-service.log 2>&1 &'
    sleep 2
    if ps aux | grep -q "[c]osmic-storage-service"; then
        echo "✓ Service started (logs: /tmp/cosmic-storage-service.log)"
    else
        echo "✗ Service failed to start. Check logs: sudo cat /tmp/cosmic-storage-service.log"
        exit 1
    fi
    echo ""

    # Step 5: Launch the UI
    echo "► Step 5/5: Launching COSMIC Ext Storage UI..."
    echo ""
    echo "==========================================="
    echo "  Development environment ready!"
    echo "  Service logs: /tmp/cosmic-storage-service.log"
    echo "==========================================="
    echo ""
    RUST_LOG=cosmic_ext_storage=debug,info,wgpu=warn,wgpu_core=warn,wgpu_hal=warn,naga=warn,iced_winit=warn,iced_wgpu=warn,i18n_embed=warn ./target/debug/cosmic-ext-storage

    # Cleanup on exit
    echo ""
    echo "App exited. Stopping service..."
    sudo pkill -f cosmic-storage-service || true
    echo "Done."

# Build all workspace crates
build:
    cargo build --workspace --locked

# Build all crates in release mode
build-release:
    cargo build --workspace --release --locked

# Run tests for all crates
test:
    cargo test --workspace --all-features --locked

# Run clippy for all crates
clippy:
    cargo clippy --workspace --all-features --locked -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Build and install system files (requires root)
install-system-files: build-release
    @echo "Installing systemd service files..."
    sudo install -Dm644 data/systemd/cosmic-storage-service.service /usr/lib/systemd/system/
    sudo install -Dm644 data/systemd/cosmic-storage-service.socket /usr/lib/systemd/system/
    @echo "Installing D-Bus policy..."
    sudo install -Dm644 data/dbus-1/system.d/org.cosmic.ext.StorageService.conf /usr/share/dbus-1/system.d/
    @echo "Installing Polkit policy..."
    sudo install -Dm644 data/polkit-1/actions/org.cosmic.ext.storage-service.policy /usr/share/polkit-1/actions/
    @echo "Installing service binary..."
    sudo install -Dm755 target/release/cosmic-storage-service /usr/bin/
    @echo "Reloading systemd..."
    sudo systemctl daemon-reload
    @echo ""
    @echo "System files installed. You can now enable the service with:"
    @echo "  sudo systemctl enable --now cosmic-storage-service.service"

# Install just the D-Bus policy for development (requires root)
install-dbus-policy:
    @echo "Installing D-Bus policy..."
    sudo install -Dm644 data/dbus-1/system.d/org.cosmic.ext.StorageService.conf /usr/share/dbus-1/system.d/
    @echo "Reloading D-Bus configuration..."
    sudo systemctl reload dbus
    @echo ""
    @echo "D-Bus policy installed. You can now run 'just service'"

# Install just the Polkit policy for development (requires root)
install-polkit-policy:
    @echo "Installing Polkit policy..."
    sudo install -Dm644 data/polkit-1/actions/org.cosmic.ext.storage-service.policy /usr/share/polkit-1/actions/
    @echo ""
    @echo "Polkit policy installed."

# Install D-Bus and Polkit policies for development (requires root)
install-dev-policies: install-dbus-policy install-polkit-policy
    @echo ""
    @echo "Development policies installed. Ready for testing!"

# Start the storage service (for development)
service: build
    #!/usr/bin/env bash
    echo "Starting storage service (requires root)..."
    sudo pkill -f cosmic-storage-service || true
    sudo RUST_LOG=storage_service=debug,info ./target/debug/cosmic-storage-service

# Start the storage service in background
service-bg: build
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting storage service in background (requires root)..."
    sudo -v
    sudo pkill -f cosmic-storage-service || true
    sudo rm -f /tmp/cosmic-storage-service.log
    sudo bash -c 'nohup env RUST_LOG=storage_service=info ./target/debug/cosmic-storage-service > /tmp/cosmic-storage-service.log 2>&1 &'
    sleep 2
    echo "Service started. Logs: /tmp/cosmic-storage-service.log"
    if ps aux | grep -q "[c]osmic-storage-service"; then
        echo "✓ Service is running"
    else
        echo "✗ Service not running. Check logs: sudo cat /tmp/cosmic-storage-service.log"
        exit 1
    fi

# Stop the storage service
stop-service:
    #!/usr/bin/env bash
    echo "Stopping storage service..."
    sudo pkill -f cosmic-storage-service || true
    echo "Service stopped"

# Start the COSMIC Ext Storage UI
app: build
    #!/usr/bin/env bash
    echo "Starting COSMIC Ext Storage UI..."
    RUST_LOG=cosmic_ext_storage=debug,info,wgpu=warn,wgpu_core=warn,wgpu_hal=warn,naga=warn,iced_winit=warn,iced_wgpu=warn,i18n_embed=warn ./target/debug/cosmic-ext-storage



# Development workflow: start service in background, then start app
dev: build
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting storage service in background (requires root)..."
    sudo -v
    sudo pkill -f cosmic-storage-service || true
    sudo rm -f /tmp/cosmic-storage-service.log
    sudo bash -c 'nohup env RUST_LOG=storage_service=info ./target/debug/cosmic-storage-service > /tmp/cosmic-storage-service.log 2>&1 &'
    sleep 2
    echo "Starting COSMIC Ext Storage UI..."
    RUST_LOG=cosmic_ext_storage=debug,info,wgpu=warn,wgpu_core=warn,wgpu_hal=warn,naga=warn,iced_winit=warn,iced_wgpu=warn,i18n_embed=warn ./target/debug/cosmic-ext-storage
    echo ""
    echo "App exited. Stopping service..."
    sudo pkill -f cosmic-storage-service || true

# Development with complete rebuild
dev-clean: clean build service-bg
    @sleep 2
    @echo "Starting COSMIC Ext Storage UI..."
    @RUST_LOG=cosmic_ext_storage=debug,info,wgpu=warn,wgpu_core=warn,wgpu_hal=warn,naga=warn,iced_winit=warn,iced_wgpu=warn,i18n_embed=warn ./target/debug/cosmic-ext-storage
    @just stop-service

# Test D-Bus interface using busctl
test-dbus:
    @echo "Testing D-Bus interface..."
    @echo "Listing service..."
    busctl --system tree org.cosmic.ext.StorageService
    @echo ""
    @echo "Introspecting BTRFS interface..."
    busctl --system introspect org.cosmic.ext.StorageService /org/cosmic/ext/StorageService/btrfs

# Test BTRFS list command via D-Bus
test-btrfs-list MOUNTPOINT="/":
    @echo "Testing BTRFS list subvolumes at {{MOUNTPOINT}}..."
    busctl --system call org.cosmic.ext.StorageService /org/cosmic/ext/StorageService/btrfs org.cosmic.ext.StorageService.Btrfs ListSubvolumes s "{{MOUNTPOINT}}"

# Monitor D-Bus signals
monitor-dbus:
    @echo "Monitoring D-Bus signals from storage service..."
    dbus-monitor --system "type='signal',sender='org.cosmic.ext.StorageService'"

# Check service status
status:
    @echo "Storage service status:"
    @systemctl status cosmic-storage-service.service || echo "Service not installed as systemd unit"
    @echo ""
    @echo "Process status:"
    @ps aux | grep cosmic-storage-service | grep -v grep || echo "Service not running"

# View service logs
logs:
    @journalctl -u cosmic-storage-service -f

# Clean build artifacts
clean:
    cargo clean

# Full development cycle: format, clippy, test, build
check: fmt clippy test build

# Install development dependencies (Debian/Ubuntu)
install-deps-deb:
    @echo "Installing development dependencies (Debian/Ubuntu)..."
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        libdbus-1-dev \
        libpolkit-gobject-1-dev \
        libbtrfs-dev \
        btrfs-progs \
        systemd \
        dbus

# Install development dependencies (Fedora)
install-deps-fedora:
    @echo "Installing development dependencies (Fedora)..."
    sudo dnf install -y \
        gcc \
        pkg-config \
        dbus-devel \
        polkit-devel \
        btrfs-progs-devel \
        btrfs-progs \
        systemd \
        dbus

# Install development dependencies (Arch)
install-deps-arch:
    @echo "Installing development dependencies (Arch)..."
    sudo pacman -S --needed \
        base-devel \
        pkg-config \
        dbus \
        polkit \
        btrfs-progs \
        systemd

# Watch for changes and rebuild
watch:
    cargo watch -x "build --workspace"

# Watch and run tests
watch-test:
    cargo watch -x "test --workspace"

# Create a debug build and run the old helper (for comparison)
run-old-helper MOUNTPOINT="/": build
    @echo "Running old helper for comparison..."
    sudo RUST_LOG=debug ./target/debug/cosmic-ext-storage-btrfs-helper list {{MOUNTPOINT}}

# Create a debug build and run the new library CLI (for comparison)
run-new-cli MOUNTPOINT="/": build
    @echo "Running new library CLI..."
    sudo RUST_LOG=debug cargo run --features cli -p storage-btrfs --bin storage-btrfs-cli -- list {{MOUNTPOINT}}
