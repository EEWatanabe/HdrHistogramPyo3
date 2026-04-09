from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import polars as pl
from polars.plugins import register_plugin_function

from hdrhistogram_pyo3._internal import __version__ as __version__

if TYPE_CHECKING:
    from hdrhistogram_pyo3.typing import IntoExprColumn

LIB = Path(__file__).parent


def pig_latinnify(expr: IntoExprColumn) -> pl.Expr:
    """Legacy example: convert words to pig latin style."""
    return register_plugin_function(
        args=[expr],
        plugin_path=LIB,
        function_name="pig_latinnify",
        is_elementwise=True,
    )


def hdr_percentile(expr: IntoExprColumn, percentile: float) -> pl.Expr:
    """
    Compute a percentile from histogram values.

    Args:
        expr: Column of numeric values to compute percentile from
        percentile: Percentile value (0-100)

    Returns:
        Polars Expr returning the percentile value
    """
    return register_plugin_function(
        args=[expr, pl.lit(percentile)],
        plugin_path=LIB,
        function_name="hdr_percentile",
        is_elementwise=False,
    )


def hdr_mean(expr: IntoExprColumn) -> pl.Expr:
    """
    Compute mean from histogram values.

    Args:
        expr: Column of numeric values

    Returns:
        Polars Expr returning the mean value
    """
    return register_plugin_function(
        args=[expr],
        plugin_path=LIB,
        function_name="hdr_mean",
        is_elementwise=False,
    )


def hdr_stats_summary(expr: IntoExprColumn) -> pl.Expr:
    """
    Compute a statistics summary string from histogram values.

    Args:
        expr: Column of numeric values

    Returns:
        Polars Expr returning a formatted string with count, min, max, mean
    """
    return register_plugin_function(
        args=[expr],
        plugin_path=LIB,
        function_name="hdr_stats_summary",
        is_elementwise=False,
    )
