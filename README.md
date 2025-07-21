# ccd - Change Change Directory

A fast directory navigation tool that uses the `locate` database to quickly find and change to directories.

## Features

- **Fast search**: Uses the `locate` database for instant directory lookup
- **Case-insensitive**: Searches are case-insensitive by default
- **Limited results**: Shows only the first 100 matches for performance
- **Directory filtering**: Automatically filters results to show only directories
- **Shell integration**: Properly changes the current shell's working directory

## Installation

1. Build the Rust binary:
   ```bash
   cargo build --release
   ```

2. Source the shell wrapper in your shell profile:
   ```bash
   # Add this to your ~/.bashrc or ~/.zshrc
   source /path/to/ccd/ccd.sh
   ```

3. Make sure your locate database is up to date:
   ```bash
   sudo updatedb
   ```

## Usage

```bash
# change into best match for "proj" (case-sensitive)
ccd Proj

# Invoke interactive mode
ccd
```

## How it works

1. The `ccd` command searches the locate database using `locate -i --limit 100 <pattern>`
2. Filters results to show only directories
3. Changes to the first directory found
4. Provides feedback about the number of directories found and which one was selected

## Requirements

- Rust (for building)
- `locate` command (usually part of `findutils` or `mlocate` package)
- Updated locate database (`sudo updatedb`)

## Examples

```bash
$ ccd tmp
Searching for directories matching: tmp
Found 19 directories in first 100 results, selected: /tmp
Changed to: /tmp

$ ccd nonexistent
Searching for directories matching: nonexistent
No files or directories found matching 'nonexistent'
```

## Shell Integration

The tool uses a shell function wrapper to properly change the current shell's directory. The Rust binary outputs the target directory path to stdout, and the shell function captures this and executes `cd` to change directories.

## Future Enhancements

- Remember frequently used directories
- Interactive selection when multiple directories match
- Configuration file for preferences
- Fuzzy matching support
