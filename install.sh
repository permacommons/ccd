#!/bin/bash

# Installer script for ccd shell function
# This script safely installs or updates the ccd function in ~/.bashrc

set -e

BASHRC="$HOME/.bashrc"
MARKER_START="# BEGIN ccd function"
MARKER_END="# END ccd function"
CCD_SCRIPT="ccd.sh"

# Check if ccd.sh exists
if [ ! -f "$CCD_SCRIPT" ]; then
    echo "Error: $CCD_SCRIPT not found in current directory"
    exit 1
fi

# Create backup of .bashrc if it exists
if [ -f "$BASHRC" ]; then
    cp "$BASHRC" "$BASHRC.backup.$(date +%Y%m%d_%H%M%S)"
    echo "Created backup: $BASHRC.backup.$(date +%Y%m%d_%H%M%S)"
fi

# Function to extract ccd function from ccd.sh (skip shebang and comments at top)
extract_ccd_function() {
    # Skip the shebang and initial comments, start from the ccd() function
    sed -n '/^ccd()/,$p' "$CCD_SCRIPT"
}

# Remove existing ccd function if present
if [ -f "$BASHRC" ] && grep -q "$MARKER_START" "$BASHRC"; then
    echo "Removing existing ccd function from $BASHRC..."
    # Use sed to remove everything between markers (inclusive)
    sed -i "/$MARKER_START/,/$MARKER_END/d" "$BASHRC"
fi

# Add the new ccd function
echo "Installing ccd function to $BASHRC..."

# Add markers and function to .bashrc
{
    echo ""
    echo "$MARKER_START"
    echo "# Shell wrapper for the ccd-pick command"
    echo "# This function should be sourced in your shell profile"
    echo ""
    extract_ccd_function
    echo ""
    echo "$MARKER_END"
} >> "$BASHRC"

echo "Successfully installed ccd function to $BASHRC"
echo ""
echo "To use ccd immediately, run:"
echo "  source ~/.bashrc"
echo ""
echo "Or start a new terminal session."
echo ""
echo "Usage:"
echo "  ccd              # Interactive mode"
echo "  ccd <pattern>    # Search for directories matching pattern"
echo "  ccd --help       # Show help"
