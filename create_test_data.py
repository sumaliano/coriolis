#!/usr/bin/env python
"""Create a sample netCDF file for testing Tanotly."""

import numpy as np
import xarray as xr


def create_test_file(output_path: str = "test_data.nc") -> None:
    """Create a sample netCDF file with various data types."""

    # Create dimensions
    time = np.arange(10)
    lat = np.linspace(-90, 90, 180)
    lon = np.linspace(-180, 180, 360)

    # Create sample data
    temperature = 15 + 8 * np.random.randn(10, 180, 360)
    precipitation = np.abs(np.random.randn(10, 180, 360)) * 10
    pressure = 1013 + 20 * np.random.randn(10, 180, 360)

    # Create dataset
    ds = xr.Dataset(
        {
            "temperature": (
                ["time", "lat", "lon"],
                temperature,
                {
                    "units": "degrees_celsius",
                    "long_name": "Air Temperature",
                    "standard_name": "air_temperature",
                    "valid_range": [-50, 50],
                },
            ),
            "precipitation": (
                ["time", "lat", "lon"],
                precipitation,
                {
                    "units": "mm/day",
                    "long_name": "Precipitation Rate",
                    "standard_name": "precipitation_flux",
                },
            ),
            "pressure": (
                ["time", "lat", "lon"],
                pressure,
                {
                    "units": "hPa",
                    "long_name": "Sea Level Pressure",
                    "standard_name": "air_pressure_at_sea_level",
                },
            ),
        },
        coords={
            "time": (
                ["time"],
                time,
                {
                    "units": "days since 2024-01-01",
                    "long_name": "Time",
                    "calendar": "gregorian",
                },
            ),
            "lat": (
                ["lat"],
                lat,
                {
                    "units": "degrees_north",
                    "long_name": "Latitude",
                    "standard_name": "latitude",
                },
            ),
            "lon": (
                ["lon"],
                lon,
                {
                    "units": "degrees_east",
                    "long_name": "Longitude",
                    "standard_name": "longitude",
                },
            ),
        },
        attrs={
            "title": "Sample Climate Data",
            "institution": "Tanotly Test Suite",
            "source": "Generated test data",
            "Conventions": "CF-1.8",
            "history": "Created for testing purposes",
        },
    )

    # Save to file
    ds.to_netcdf(output_path)
    print(f"✓ Created test file: {output_path}")
    print(f"  - Dimensions: time={len(time)}, lat={len(lat)}, lon={len(lon)}")
    print(f"  - Variables: temperature, precipitation, pressure")
    print(f"\nYou can now run:")
    print(f"  ./run.sh {output_path}")


if __name__ == "__main__":
    import sys

    output = sys.argv[1] if len(sys.argv) > 1 else "test_data.nc"
    create_test_file(output)
