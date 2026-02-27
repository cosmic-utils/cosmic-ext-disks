# storage-testing

`storage-testing` provides two local binaries for integration and manual lab workflows:

- `harness`: runs integration tests directly on host.
- `lab`: prepares loop-backed disk images on host for manual CRUD testing through the COSMIC UI.

## Commands

### harness

Host harness commands require root privileges for loop/LVM/mdadm operations.

```bash
just harness
sudo cargo run -p storage-testing --bin harness -- run --suite logical
sudo cargo run -p storage-testing --bin harness -- cleanup
```

### lab

Lab specs are resolved by spec name (no extension) from `resources/lab-specs`.

```bash
just lab 2disk
just lab 3disk

cargo run -p storage-testing --bin lab -- image prepare 2disk
cargo run -p storage-testing --bin lab -- image attach 2disk
cargo run -p storage-testing --bin lab -- image mount 2disk
cargo run -p storage-testing --bin lab -- image unmount 2disk
cargo run -p storage-testing --bin lab -- image detach 2disk
cargo run -p storage-testing --bin lab -- image destroy 2disk
```

All mutating `lab` commands are destructive by default. Add `--dry-run` to preview commands.
