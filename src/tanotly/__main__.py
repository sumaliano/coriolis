"""Main entry point for Tanotly."""

import sys
from pathlib import Path

from tanotly.app import TanotlyApp


def main() -> None:
    """Main entry point."""
    file_path = None

    # Check for file argument
    if len(sys.argv) > 1:
        file_path = sys.argv[1]
        # Validate file exists
        if not Path(file_path).exists():
            print(f"Error: File not found: {file_path}")
            sys.exit(1)

    # Run the Textual app (dual-pane viewer)
    app = TanotlyApp(file_path=file_path)
    app.run()


if __name__ == "__main__":
    main()
