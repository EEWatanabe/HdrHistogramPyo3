# AGENTS.md - HdrHistogramPyo3 Development Guide

## Project Overview

**HdrHistogramPyo3** is a PyO3-based Python extension that wraps Rust implementations as Polars expression plugins. It currently exposes:
- **`pig_latinnify`**: Legacy string transformation example
- **HdrHistogram functions**: High-performance histogram statistics for latency/performance analysis

The project uses Rust's `hdrhistogram` crate (v7) to provide efficient percentile and statistical computations on numeric data through Polars expressions.

- **Architecture**: Rust (via PyO3) + Polars plugin system + Python wrapper + hdrhistogram crate
- **Build System**: Maturin (Rust-to-Python compiler)
- **Key Stack**: Rust 2021 (nightly), Python 3.8+, Polars 0.53+, hdrhistogram 7.x

## Critical Build & Workflow Commands

### Setup & Installation
```bash
make venv              # Create virtual environment with dependencies
make install           # Build extension in debug mode (maturin develop)
make install-release   # Build optimized extension
make run               # Build + run example (see run.py)
```

**Important**: The `unset CONDA_PREFIX` in Makefile is intentional—it prevents conda environment conflicts with maturin. Preserve this when modifying build targets.

### Code Quality & Testing
```bash
make pre-commit        # Full validation: cargo fmt, clippy, ruff, mypy, then pytest
make test              # Run pytest suite on tests/
```

**Note**: Rust uses nightly toolchain (see rust-toolchain.toml). Pre-commit runs `cargo +nightly fmt` explicitly.

## Architecture: Rust → Polars → Python

### Three-Layer Design

1. **Rust Layer** (`src/lib.rs`, `src/expressions.rs`)
   - PyO3 module `_internal` (compiled to `.so`)
   - Polars expressions via `#[polars_expr]` macro
   - HdrHistogram integration for efficient statistical computation
   - Direct value processing using `StringChunked.apply_into_string_amortized()` (strings) or by accumulating values into histograms (numerics)
   - Uses `PolarsAllocator` for memory management
   - Examples: 
     - `pig_latinnify` transforms "hello" → "ellohay" (string example)
     - `hdr_mean`, `hdr_percentile` compute stats from numeric columns (HdrHistogram examples)

2. **Python Wrapper** (`hdrhistogram_pyo3/__init__.py`)
   - Function signatures like `pig_latinnify(expr: IntoExprColumn) → pl.Expr`
   - HdrHistogram functions with parameters: `hdr_percentile(expr, percentile_value)`
   - Calls `register_plugin_function()` to bridge Polars + Rust
   - Marks functions `is_elementwise=True` for element-wise operations, `is_elementwise=False` for aggregations
   - Plugin path discovered via `LIB = Path(__file__).parent`

3. **Type Definitions** (`hdrhistogram_pyo3/typing.py`)
   - `IntoExprColumn`: Union of `pl.Expr`, `str`, or `pl.Series`
   - Polars type aliases for cross-version compatibility (3.10+ TypeAlias support)

### Extension Module Layout
- `.so` files are auto-generated (don't edit): naming varies by Python version
- `.pyi` stub file (`_internal.pyi`) documents exported symbols: only `__version__` currently
- Module name configured in `pyproject.toml` as `hdrhistogram_pyo3._internal`

## HdrHistogram Integration Pattern

The `hdrhistogram` crate is integrated to provide aggregation operations on numeric data. Key pattern:

```rust
#[polars_expr(output_type = Float64)]  // Specify return type
fn hdr_mean(inputs: &[Series]) -> PolarsResult<Series> {
    let values = inputs[0].f64()?;  // Extract numeric column
    
    // Create a new histogram
    let mut histogram: Histogram<u64> = Histogram::new(3)
        .map_err(|e| PolarsError::ComputeError(format!(...).into()))?;
    
    // Accumulate all values into the histogram
    for val in values.iter().flatten() {
        let val_u64 = val as u64;  // Convert to u64 (hdrhistogram requirement)
        histogram.record(val_u64).map_err(|e| { ... })?;
    }
    
    // Compute statistic from histogram
    let result = histogram.mean();
    
    // Return as single-value Series
    let output = Float64Chunked::from_slice(
        PlSmallStr::from_str("mean"),
        &[result]
    );
    Ok(output.into_series())
}
```

**Key points for HdrHistogram functions**:
- Accept numeric (f64) input columns
- Record values as `u64` (convert `f64 as u64`)
- Create `Histogram<u64>` with appropriate significant figures parameter (typically 3)
- Return results as single-value Series (aggregation, not element-wise)
- Mark in Python wrapper as `is_elementwise=False`

## Adding New Expressions

### String Transformation Pattern (Element-wise)
```rust
#[polars_expr(output_type=String)]
fn my_transform(inputs: &[Series]) -> PolarsResult<Series> {
    let ca: &StringChunked = inputs[0].str()?;
    let out: StringChunked = ca.apply_into_string_amortized(|value: &str, output: &mut String| {
        write!(output, "{}", value).unwrap()
    });
    Ok(out.into_series())
}
```

### Aggregation Pattern (HdrHistogram)
Follow the pattern shown above, mark as `is_elementwise=False` in Python.

## Key Integration Points & Dependencies

| Layer | Dependency | Version | Purpose |
|-------|----------|---------|---------|
| Rust → Python | `pyo3` | 0.27.0 | PyO3 bindings |
| Rust → Polars | `pyo3-polars` | 0.26.0 | Polars expression macros + allocator |
| Rust → Stats | `hdrhistogram` | 7.x | High-resolution histogram statistics |
| Python → Polars | `polars` | 0.53.0 | DataFrame operations, plugin system |
| Build | `maturin` | ≥1.0,<2.0 | Compiles Rust to Python wheel |
| Quality | `ruff`, `mypy` | (latest) | Linting & type checking |

**Allocator**: `PolarsAllocator` must be set as `#[global_allocator]` in `lib.rs` to align Rust memory with Polars' expectations.

## Testing Pattern

Tests live in `tests/test_pig_latinnify.py` and follow this structure:

### Element-wise Operations
```python
df = pl.DataFrame({"input_col": ["value1", "value2"]})
result = df.with_columns(output=expression("input_col"))
assert result.equals(expected_df)
```

### Aggregations (HdrHistogram)
```python
df = pl.DataFrame({"values": [10.0, 20.0, 30.0, 40.0, 50.0]})
result = df.select(statistic=hdr_mean("values"))
assert isinstance(result["statistic"][0], float)
```

Run with `make test` (pytest auto-discovers).

## Development Workflows

### Adding HdrHistogram Functions
1. Define Rust function with `#[polars_expr(output_type=...)]` in `src/expressions.rs`
2. Create histogram, accumulate values, compute result
3. Export in Python at `hdrhistogram_pyo3/__init__.py` with proper `is_elementwise=False`
4. Add tests following aggregation pattern
5. Run `make install && make test`

### Modifying Rust Code
1. Edit `src/*.rs`
2. Run `make install` to recompile
3. Test with `make run` or `make test`
4. Before commit: `make pre-commit` (includes `cargo +nightly fmt`, clippy)

### Modifying Python Wrapper
1. Edit `hdrhistogram_pyo3/*.py`
2. Run `make pre-commit` (ruff format + mypy type check)
3. No recompilation needed unless changing `__init__.py` imports

### Debugging Build Issues
- `maturin develop` output in `target/debug/` shows compilation errors
- Check Rust toolchain: `rustup show` (must be nightly)
- Verify `CONDA_PREFIX` is unset: `echo $CONDA_PREFIX`
- For plugin symbol errors ("undefined symbol: _polars_plugin_field_*"), rebuild from scratch: `rm -rf target hdrhistogram_pyo3/_internal*.so && make install`

## Code Style & Conventions

- **Rust**: Nightly toolchain with `cargo fmt --all` (enforced via pre-commit)
- **Python**: Ruff formatter & linter with mypy strict type checking
- **Naming**: Snake case for all functions (e.g., `pig_latinnify`, `hdr_mean`)
- **Polars Macros**: `#[polars_expr(output_type=...)]` declares return type
- **Error Handling**: Use `PolarsResult<T>` (Polars' Result type) in Rust
- **Series Creation**: Use `PlSmallStr::from_str("name")` for column names (not `&str`)
- **Type Conversion**: For HdrHistogram, convert `f64` to `u64` explicitly with `val as u64`

## Common Gotchas

1. **Conda environment conflicts**: Always `unset CONDA_PREFIX` before maturin builds (already in Makefile)
2. **Nightly requirement**: Rust toolchain is nightly; `cargo +nightly fmt` is explicit in pre-commit
3. **Plugin path discovery**: `LIB = Path(__file__).parent` must point to where `.so` lives after installation
4. **Type stubs**: Keep `.pyi` file in sync with exported symbols from `lib.rs`
5. **Polars version lock**: `pyo3-polars` 0.26.0 requires `polars` 0.53.0; upgrading one may break the other
6. **Data type mismatches**: HdrHistogram functions expect `f64` input columns; ensure data is cast correctly
7. **Plugin symbol loading**: If you see "undefined symbol: _polars_plugin_field_*" errors, the `.so` file wasn't regenerated. Clean and rebuild with `rm -rf target hdrhistogram_pyo3/_internal*.so && make install`
8. **Element-wise vs aggregation**: Set `is_elementwise=True` for per-element operations, `False` for aggregations that collapse rows



