#!/bin/bash

# Test script to verify cdd functionality
echo "Current directory before: $(pwd)"

# Source the cdd function
source ./cdd.sh

# Test the direct search functionality
echo "Testing cdd tmp..."
cdd tmp

echo "Current directory after direct search: $(pwd)"

# Test help
echo -e "\nTesting help:"
cdd --help

echo -e "\nInteractive mode is available with just 'cdd' (no arguments)"
echo "This will open a full-screen interface where you can:"
echo "- Type to search for directories"
echo "- Use ↑/↓ to navigate results"
echo "- Press Enter to select"
echo "- Press q or Esc to quit"
