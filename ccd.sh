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
        output=$(ccd-pick -i 3>&1 >/dev/tty 2>&1)
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
        ccd-pick "$@"
        return
    fi
    
    # Capture the output from the ccd binary
    local output
    output=$(ccd-pick "$@" 2>/dev/null)
    local exit_code=$?
    
    if [ $exit_code -eq 0 ] && [ -n "$output" ] && [ -d "$output" ]; then
        # Successfully found a directory, change to it
        cd "$output"
        echo "Changed to: $output"
    else
        # Show error output or help
        ccd-pick "$@"
    fi
}
