#![allow(clippy::unused_unit)]
use std::fmt::Write;

use hdrhistogram::Histogram;
use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

/// Legacy example: transforms first character to end of word with "ay"
#[polars_expr(output_type = String)]
fn pig_latinnify(inputs: &[Series]) -> PolarsResult<Series> {
    let ca: &StringChunked = inputs[0].str()?;
    let out: StringChunked = ca.apply_into_string_amortized(|value: &str, output: &mut String| {
        if let Some(first_char) = value.chars().next() {
            write!(output, "{}{}ay", &value[1..], first_char).unwrap()
        }
    });
    Ok(out.into_series())
}

/// Compute percentile from a column of numeric values
/// Returns the specified percentile value from the input histogram
#[polars_expr(output_type = Float64)]
fn hdr_percentile(inputs: &[Series]) -> PolarsResult<Series> {
    let values = inputs[0].f64()?;
    let percentile = inputs[1].f64()?;

    let percentile_val = percentile
        .get(0)
        .ok_or_else(|| PolarsError::ComputeError("Percentile value required".into()))?;

    // Create histogram from values
    let mut histogram: Histogram<u64> = Histogram::new(3).map_err(|e| {
        PolarsError::ComputeError(format!("Failed to create histogram: {}", e).into())
    })?;

    // Record all values in histogram
    for val in values.iter().flatten() {
        let val_u64 = val as u64;
        histogram.record(val_u64).map_err(|e| {
            PolarsError::ComputeError(format!("Failed to record value: {}", e).into())
        })?;
    }

    // Compute percentile
    let result_val = histogram.value_at_percentile(percentile_val);

    // Create output series with single value
    let output =
        Float64Chunked::from_slice(PlSmallStr::from_str("percentile"), &[result_val as f64]);
    Ok(output.into_series())
}

/// Compute mean from a column of numeric values using HdrHistogram
#[polars_expr(output_type = Float64)]
fn hdr_mean(inputs: &[Series]) -> PolarsResult<Series> {
    let values = inputs[0].f64()?;

    // Create histogram from values
    let mut histogram: Histogram<u64> = Histogram::new(3).map_err(|e| {
        PolarsError::ComputeError(format!("Failed to create histogram: {}", e).into())
    })?;

    // Record all values in histogram
    for val in values.iter().flatten() {
        let val_u64 = val as u64;
        histogram.record(val_u64).map_err(|e| {
            PolarsError::ComputeError(format!("Failed to record value: {}", e).into())
        })?;
    }

    // Compute mean
    let mean_val = histogram.mean();

    // Create output series with single value
    let output = Float64Chunked::from_slice(PlSmallStr::from_str("mean"), &[mean_val]);
    Ok(output.into_series())
}

/// Compute statistics: count, min, max, mean from histogram values
#[polars_expr(output_type = String)]
fn hdr_stats_summary(inputs: &[Series]) -> PolarsResult<Series> {
    let values = inputs[0].f64()?;

    // Create histogram from values
    let mut histogram: Histogram<u64> = Histogram::new(3).map_err(|e| {
        PolarsError::ComputeError(format!("Failed to create histogram: {}", e).into())
    })?;

    // Record all values in histogram
    let mut count = 0i64;
    for val in values.iter().flatten() {
        let val_u64 = val as u64;
        histogram.record(val_u64).map_err(|e| {
            PolarsError::ComputeError(format!("Failed to record value: {}", e).into())
        })?;
        count += 1;
    }

    // Create summary string
    let summary = format!(
        "count={}, min={}, max={}, mean={:.2}",
        count,
        histogram.min(),
        histogram.max(),
        histogram.mean()
    );

    let output = StringChunked::from_slice(PlSmallStr::from_str("stats"), &[summary.as_str()]);
    Ok(output.into_series())
}
