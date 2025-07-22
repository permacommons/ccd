# ccd - Change Change Directory

A fast, intelligent directory navigation tool that uses the `locate` database to quickly find and change to directories. Features frequency tracking and an interactive TUI for easy directory selection.

## Features

- **Fast search**: Uses the `locate` database for instant directory lookup
- **Interactive mode**: Beautiful TUI interface for browsing and selecting directories
- **Frequency tracking**: Remembers and prioritizes frequently used directories
- **Smart sorting**: Results sorted by usage frequency, then by path length
- **Keyboard navigation**: Full keyboard support for easy navigation
- **Safe installation**: Automated installer with backup and update support
- **Shell integration**: Properly changes the current shell's working directory

## Installation

### Quick Install

1. Install the Rust binary to your PATH:
   ```bash
   cargo install --path . --locked
   ```

2. Ensure `~/.cargo/bin` is in your PATH (add to ~/.bashrc if needed):
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

3. Run the installer to add the shell function to your ~/.bashrc:
   ```bash
   ./install.sh
   ```

4. Reload your shell configuration:
   ```bash
   source ~/.bashrc
   ```

5. Make sure your locate database is up to date:
   ```bash
   sudo updatedb
   ```

### Manual Installation

If you prefer manual installation, you can source the shell wrapper directly:
```bash
# Add this to your ~/.bashrc or ~/.zshrc
source /path/to/ccd/ccd.sh
```

## Usage

### Interactive Mode (Recommended)
```bash
# Launch interactive directory picker
ccd
```

**Interactive Mode Controls:**
- Type to search for directories
- `↑/↓`: Navigate through results
- `PgUp/PgDn`: Fast navigation (10 items at a time)
- `Home/End`: Jump to first/last result
- `Enter`: Select directory and change to it
- `Delete`: Reset frequency count for selected directory
- `q/Esc`: Quit without changing directory

### Direct Search
```bash
# Change to best match for "proj"
ccd proj

# Show help
ccd --help
```

## How it works

### Search Process
1. The `ccd-pick` binary searches the locate database using `locate --limit 100 <pattern>`
2. Filters results to show only directories
3. Loads frequency data from `~/.kcd_frequency`
4. Sorts results by usage frequency (most used first), then by path length
5. In direct mode: changes to the first directory found
6. In interactive mode: presents a TUI for selection

### Frequency Tracking
- Each time you select a directory, its usage count is incremented
- Frequently used directories appear at the top of search results
- You can reset frequency counts using the `Delete` key in interactive mode
- Frequency data is stored in `~/.kcd_frequency`

### Shell Integration
The tool uses a shell function wrapper (`ccd`) that calls the Rust binary (`ccd-pick`) and properly changes the current shell's directory. The binary outputs the target directory path, and the shell function captures this and executes `cd`.

## Examples

### Interactive Mode
```bash
$ ccd
# Opens TUI interface - type "proj" to search
# Use arrow keys to navigate, Enter to select
```

### Direct Mode
```bash
$ ccd tmp
Searching for directories matching: tmp
Found 19 directories in first 100 results, selected: /tmp (used 5 times)
Changed to: /tmp

$ ccd nonexistent
Searching for directories matching: nonexistent
No directories found matching 'nonexistent'
```

## Requirements

- **Rust** (for building the binary)
- **locate** command (usually part of `findutils` or `mlocate` package)
- **Updated locate database** (`sudo updatedb`)
- **Bash** (for the shell wrapper)

## Project Structure

- `src/main.rs` - Main Rust application (`ccd-pick` binary)
- `ccd.sh` - Shell wrapper function
- `install.sh` - Automated installer script
- `Cargo.toml` - Rust project configuration

## Development

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Install to ~/.cargo/bin
cargo install --path . --locked
```

### Testing
```bash
# Run tests
cargo test

# Test interactive mode (after cargo install)
ccd-pick -i

# Test direct search (after cargo install)
ccd-pick "search-term"

# Or test with local build
./target/debug/ccd-pick -i
./target/debug/ccd-pick "search-term"
```

## Contributing

See [AGENTS.md](AGENTS.md) for contribution guidelines. We welcome contributions from both humans and AI assistants!

## License

This project is open source. Please see the license file for details.
