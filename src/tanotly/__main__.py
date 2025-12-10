"""Main entry point for Tanotly."""

import sys
import logging
from pathlib import Path

from tanotly.app import TanotlyApp


def setup_logging(log_level: str = "INFO") -> None:
    """Configure logging for the application.
    
    Logs are sent to:
    1. Textual console (if running with `textual console`)
    2. File: tanotly.log
    
    Note: We only use file logging to avoid interfering with Textual's display.
    Use `textual console` in a separate terminal to see logs in real-time.
    
    Args:
        log_level: Logging level (DEBUG, INFO, WARNING, ERROR)
    """
    # Convert string to logging level
    numeric_level = getattr(logging, log_level.upper(), logging.INFO)
    
    # Configure root logger with ONLY file handler
    # DO NOT use StreamHandler as it interferes with Textual's display
    logging.basicConfig(
        level=numeric_level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            # File handler - writes to tanotly.log
            logging.FileHandler('tanotly.log', mode='w'),
        ],
        force=True  # Override any existing configuration
    )
    
    # Set specific loggers to appropriate levels
    logging.getLogger('tanotly').setLevel(numeric_level)
    
    # Reduce noise from external libraries
    logging.getLogger('asyncio').setLevel(logging.WARNING)
    logging.getLogger('textual').setLevel(logging.WARNING)


def main() -> None:
    """Main entry point."""
    # Setup logging first
    # Change to "DEBUG" for more verbose logging
    setup_logging(log_level="DEBUG")
    
    logger = logging.getLogger(__name__)
    logger.info("Starting Tanotly")
    
    file_path = None

    # Check for file argument
    if len(sys.argv) > 1:
        file_path = sys.argv[1]
        # Validate file exists
        if not Path(file_path).exists():
            print(f"Error: File not found: {file_path}")
            sys.exit(1)
        logger.info(f"Loading file: {file_path}")

    # Run the Textual app (dual-pane viewer)
    app = TanotlyApp(file_path=file_path)
    app.run()
    
    logger.info("Tanotly exited")


if __name__ == "__main__":
    main()
