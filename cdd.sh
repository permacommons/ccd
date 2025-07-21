#!/bin/bash

# Shell wrapper for the cdd command
# This function should be sourced in your shell profile (e.g., ~/.bashrc or ~/.zshrc)
# Usage: source cdd.sh

cdd() {
    if [ $# -eq 0 ]; then
        # No arguments - enter interactive mode
        # Use file descriptor 3 to capture output while allowing TUI to use stdin/stdout/stderr
        local output
        exec 3>&1  # Save stdout to fd 3
        output=$(./target/debug/cdd -i 3>&1 >/dev/tty 2>&1)
        local exit_code=$?
        exec 3>&-  # Close fd 3
        
        if [ $exit_code -eq 0 ] && [ -n "$output" ] && [ -d "$output" ]; then
            # Successfully selected a directory, change to it
            cd "$output"
            echo "Changed to: $output"
        elif [ $exit_code -eq 1 ]; then
            # User quit without selecting, don't change directory
            echo "Selection cancelled"
        fi
        return
    elif [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
        # Show help
        ./target/debug/cdd "$@"
        return
    fi
    
    # Capture the output from the cdd binary
    local output
    output=$(./target/debug/cdd "$@" 2>/dev/null)
    local exit_code=$?
    
    if [ $exit_code -eq 0 ] && [ -n "$output" ] && [ -d "$output" ]; then
        # Successfully found a directory, change to it
        cd "$output"
        echo "Changed to: $output"
    else
        # Show error output or help
        ./target/debug/cdd "$@"
    fi
}

# Alternative installation method - create a symlink in PATH
install_cdd() {
    local install_dir="$HOME/.local/bin"
    local script_path="$install_dir/cdd"
    
    # Create the directory if it doesn't exist
    mkdir -p "$install_dir"
    
    # Create the wrapper script
    cat > "$script_path" << 'EOF'
#!/bin/bash
# cdd wrapper script

if [ $# -eq 0 ] || [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    # Show help or handle no arguments
    cdd-binary "$@"
    return
fi

# Capture the output from the cdd binary
output=$(cdd-binary "$@" 2>/dev/null)
exit_code=$?

if [ $exit_code -eq 0 ] && [ -n "$output" ] && [ -d "$output" ]; then
    # Successfully found a directory, change to it
    cd "$output"
    echo "Changed to: $output"
else
    # Show error output or help
    cdd-binary "$@"
fi
EOF
    
    chmod +x "$script_path"
    echo "Installed cdd wrapper to $script_path"
    echo "Make sure $install_dir is in your PATH"
    echo "You'll also need to copy the cdd binary to $install_dir/cdd-binary"
}
