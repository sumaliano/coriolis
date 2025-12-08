"""Data table component for displaying array data."""

import numpy as np
from textual.widgets import DataTable


class ArrayDataTable:
    """Handles population and formatting of array data tables."""

    @staticmethod
    def format_value(val) -> str:
        """Format a value for display in the table.

        Args:
            val: Value to format

        Returns:
            Formatted string representation
        """
        if isinstance(val, float):
            if np.isnan(val):
                return "NaN"
            if abs(val) >= 1e6 or (abs(val) < 1e-3 and val != 0):
                return f"{val:.3e}"
            return f"{val:.4g}"
        return str(val)

    @staticmethod
    async def populate_table(
        table: DataTable,
        data: np.ndarray,
        dim_names: tuple = (),
        max_rows: int = 500,
        max_cols: int = 50
    ) -> None:
        """Populate the array data table.

        Args:
            table: DataTable widget to populate
            data: Array data to display
            dim_names: Names of dimensions
            max_rows: Maximum rows to display
            max_cols: Maximum columns to display
        """
        table.cursor_type = "cell"
        table.zebra_stripes = True

        if table.row_count > 0:
            table.clear(columns=True)

        if data.ndim == 1:
            table.add_column("Index", width=8)
            table.add_column("Value", width=20)
            for i, val in enumerate(data[:max_rows]):
                table.add_row(str(i), ArrayDataTable.format_value(val))
            if len(data) > max_rows:
                table.add_row("...", f"({len(data) - max_rows} more)")

        elif data.ndim == 2:
            rows, cols = data.shape
            display_cols = min(cols, max_cols)
            display_rows = min(rows, max_rows)

            # Get the row dimension name (last-2 dimension from original data)
            ndim = len(dim_names)
            row_dim_name = dim_names[ndim - 2] if ndim >= 2 else "row"

            table.add_column(row_dim_name, width=8)
            for j in range(display_cols):
                table.add_column(str(j), width=10)
            if cols > max_cols:
                table.add_column("...", width=5)

            for i in range(display_rows):
                row = [str(i)]
                for j in range(display_cols):
                    row.append(ArrayDataTable.format_value(data[i, j]))
                if cols > max_cols:
                    row.append("...")
                table.add_row(*row)

            if rows > max_rows:
                table.add_row("...", *["..."] * (display_cols + (1 if cols > max_cols else 0)))
