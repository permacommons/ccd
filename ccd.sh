#!/bin/bash

# Shell wrapper for the ccd-pick command
# This function should be sourced in your shell profile (e.g., ~/.bashrc or ~/.zshrc)
# Usage: source ccd.sh

ccd() {
    if [ $# -eq 0 ]; then
        # No arguments - enter interactive mode
        # Use file descriptor 3 to capture output while allowing TUI to use stdin/stdout/stderr
        local output
        exec 3>&1  # Save stdout to fd 3
        output=$(./target/debug/ccd -i 3>&1 >/dev/tty 2>&1)
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
        ./target/debug/ccd "$@"
        return
    fi
    
    # Capture the output from the ccd binary
    local output
    output=$(./target/debug/ccd "$@" 2>/dev/null)
    local exit_code=$?
    
    if [ $exit_code -eq 0 ] && [ -n "$output" ] && [ -d "$output" ]; then
        # Successfully found a directory, change to it
        cd "$output"
        echo "Changed to: $output"
    else
        # Show error output or help
        ./target/debug/ccd "$@"
    fi
}

# Alternative installation method - create a symlink in PATH
install_ccd() {
    local install_dir="$HOME/.local/bin"
    local script_path="$install_dir/ccd"
    
    # Create the directory if it doesn't exist
    mkdir -p "$install_dir"
    
    # Create the wrapper script
    cat > "$script_path" << 'EOF'
#!/bin/bash
# ccd wrapper script

if [ $# -eq 0 ] || [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    # Show help or handle no arguments
    ccd-binary "$@"
    return
fi

# Capture the output from the ccd binary
output=$(ccd-binary "$@" 2>/dev/null)
exit_code=$?

if [ $exit_code -eq 0 ] && [ -n "$output" ] && [ -d "$output" ]; then
    # Successfully found a directory, change to it
    cd "$output"
    echo "Changed to: $output"
else
    # Show error output or help
    ccd-binary "$@"
fi
EOF
    
    chmod +x "$script_path"
    echo "Installed ccd wrapper to $script_path"
    echo "Make sure $install_dir is in your PATH"
    echo "You'll also need to copy the ccd binary to $install_dir/ccd-binary"
}
