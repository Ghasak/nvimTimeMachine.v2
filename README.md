# nvimTimeMachine

`nvimTimeMachine` is a Rust-based CLI tool for creating, listing, and restoring
time capsules of your Neovim configuration, cache, and data. It packs your
`~/.local/share/nvim`, `~/.config/nvim`, and `~/.cache/nvim` folders into a
timestamped ZIP archive and provides an interactive restore flow.

## Features

- 📦 **Create** a new time capsule of your Neovim environment
- 📋 **List** all existing capsules with colored indices
- 🔄 **Restore** a capsule interactively (select via arrow keys)
- 💾 **Backup or delete** your current Neovim directories before restoring
- 🌈 **Colored** CLI help and output

## Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/YOUR_USERNAME/nvimTimeMachine.git
   cd nvimTimeMachine
   ```

2. **Add dependencies** (ensure `Cargo.toml` contains):

   ```toml
   [dependencies]
   clap = { version = "4.1", features = ["derive"] }
   indicatif = "0.17"
   walkdir = "2.3"
   zip = "0.6"
   dirs = "4.0"
   chrono = { version = "0.4", features = ["local"] }
   dialoguer = "0.10"
   ```

3. **Build**:

   ```bash
   cargo build --release
   ```

4. **(Optional)** Install to your `$PATH`:

   ```bash
   cargo install --path .
   ```

## Usage

Run the CLI with one of the available flags:

```bash
nvimTimeMachine [OPTIONS]
```

### Flags

- `-c`, `--create_capsule`
  Create a new ZIP time capsule of Neovim directories.

- `-l`, `--list_capsules`
  List all existing capsules with colored indices.

- `-r`, `--restore_capsule`
  Restore a selected capsule interactively.

- `-h`, `--help`
  Show help information (in color).

- `-V`, `--version`
  Show the current version.

### Examples

- **Create** a new time capsule:

  ```bash
  nvimTimeMachine --create_capsule
  ```

- **List** available capsules:

  ```bash
  nvimTimeMachine --list_capsules
  ```

  ```text
  [ ]:(1): "nvim_backup_20250513120000.zip"
  [ ]:(2): "nvim_backup_20250508180000.zip"
  ```

- **Restore** a capsule:

  ```bash
  nvimTimeMachine --restore_capsule
  ```

  1. Use arrow keys to select the desired capsule.
  2. Confirm whether to back up (rename) or delete current Neovim dirs.
  3. Wait for the restoration progress to complete.

## Contributing

1. Fork the repo
2. Create a feature branch
3. Submit a pull request

## License

MIT © Ghasak Ibrahim
