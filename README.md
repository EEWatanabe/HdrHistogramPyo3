# HdrHistogramPyo3

A PyO3-based Python extension that wraps Rust implementations as Polars expression plugins, featuring high-performance histogram statistics for latency and performance analysis.

## Features

- **HdrHistogram Statistics**: Compute percentiles, mean, and summaries from numeric columns using the efficient `hdrhistogram` Rust crate
- **String Transformations**: Element-wise string operations (e.g., `pig_latinnify`)
- **Polars Integration**: Use as native Polars expressions within DataFrames
- **High Performance**: Direct Rust compilation via PyO3 with optimized memory management

## Quick Start

```python
import polars as pl
from hdrhistogram_pyo3 import hdr_mean, hdr_percentile, hdr_stats_summary

# Create a DataFrame with latency measurements
df = pl.DataFrame({"latency_ms": [10.0, 15.0, 20.0, 25.0, 30.0]})

# Compute statistics using HdrHistogram
result = df.select(
    mean=hdr_mean("latency_ms"),
    p50=hdr_percentile("latency_ms", 50.0),
    p95=hdr_percentile("latency_ms", 95.0),
    summary=hdr_stats_summary("latency_ms"),
)

print(result)
```

## Development

See [AGENTS.md](./AGENTS.md) for comprehensive development guide including:
- Build and test workflows
- Architecture and design patterns
- Adding new expressions
- Common gotchas and troubleshooting

### Quick Commands

```bash
make venv              # Create virtual environment
make install           # Build extension
make test              # Run tests
make run               # Run example
make pre-commit        # Full validation (fmt + lint + test)
```

## Stack

- **Rust 2021** (nightly) with `hdrhistogram` v7
- **PyO3** 0.27 for Python bindings
- **Polars** 0.53+ with plugin system
- **Maturin** for build orchestration

