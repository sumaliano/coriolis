#!/bin/bash
# Installation script for Tanotly

set -e

echo "Installing Tanotly..."

# Check Python version
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "Python version: $python_version"

# Install in development mode
echo "Installing dependencies..."
pip install -e .

echo ""
echo "✓ Installation complete!"
echo ""
echo "You can now run Tanotly with:"
echo "  tanotly /path/to/your/data.nc"
echo ""
echo "Or:"
echo "  python -m tanotly /path/to/your/data.nc"
echo ""
echo "For help, press 'q' in the application or check README.md"
