#![allow(clippy::unused_unit)]
use std::fmt::Write;

use core::time;
use std::convert::From;
use std::fs::File;
use std::io::{Cursor, Read};
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::prelude::BASE64_STANDARD;
use chrono::{DateTime, TimeZone, Utc};
use hdrhistogram::iterators::IterationValue;
use hdrhistogram::serialization::interval_log::{IntervalLogIterator, LogEntry, LogIteratorError};
use hdrhistogram::serialization::{DeserializeError, Deserializer};
use hdrhistogram::{CreationError, Histogram};
use polars::prelude::*;
use pyo3::exceptions::{PyMemoryError, PyValueError};
use pyo3::{pyclass, pymethods, PyErr, PyResult};
use pyo3_polars::derive::polars_expr;

pub fn from_decoded_histogram(decoded_histogram:Vec<u8>) -> Result<Histogram<u64>, DeserializeError> {
    Deserializer::new().deserialize(&mut Cursor::new(decoded_histogram))
}

fn from_duration(duration:Duration) -> DateTime<Utc> {
    Utc.timestamp_nanos(duration.as_nanos().try_into().unwrap())
}

#[pyclass(name="HdrHistogram", frozen)]
pub struct HdrHistogram {
    hist: Arc<Mutex<Histogram<u64>>>,
}

#[pyclass(name="HdrIntervalLogHistogram", frozen)]
#[derive(Clone)]
pub struct HdrIntervalLogHistogram {
    tag: Option<String>,
    start_timestamp: DateTime<Utc>,
    duration: Duration,
    max: f64,
    decoded_histogram: Vec<u8>,
}

#[pyclass(name="HistogramLogReader")]
pub struct HistogramLogReader {
    hists: Vec<HdrIntervalLogHistogram>,
}

#[pyclass(name="HistogramLogWriter", frozen)]
pub struct HistogramLogWriter {}

#[pyclass(name="PyIterationValue")]
pub struct PyIterationValue(IterationValue<u64>);

#[pymethods]
impl PyIterationValue {
    pub fn value_iterated_to(&self) -> u64 {
        self.0.value_iterated_to()
    }

    pub fn quantile(&self) -> f64 { self.0.quantile() }

    pub fn quantile_iterated_to(&self) -> f64 { self.0.quantile_iterated_to() }

    pub fn count_at_value(&self) -> u64 {self.0.count_at_value()}

    pub fn count_since_last_iteration(&self) -> u64 { self.0.count_since_last_iteration() }
}

struct IntervalLogBufHolder {
    data: Vec<u8>,
}

impl<'a> IntoIterator for &'a IntervalLogBufHolder {
    type Item = Result<LogEntry<'a>, LogIteratorError>;
    type IntoIter = IntervalLogIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        IntervalLogIterator::new(self.data.as_slice())
    }
}

fn load_interval_log_from_file<'a>(path: &Path) -> IntervalLogBufHolder {
    let mut buf = Vec::new();
    let _ = File::open(path).unwrap().read_to_end(&mut buf).unwrap();
    IntervalLogBufHolder { data: buf }
}
#[pymethods]
impl HdrHistogram {
    pub fn distinct_values(&self) -> usize {
        self.hist.lock().unwrap().distinct_values()
    }

    pub fn low(&self) -> u64 {
        self.hist.lock().unwrap().low()
    }

    pub fn high(&self) -> u64 {
        self.hist.lock().unwrap().high()
    }

    pub fn sigfig(&self) -> u8 {
        self.hist.lock().unwrap().sigfig()
    }

    pub fn count(&self) -> u64 {
        self.hist.lock().unwrap().len()
    }

    pub fn len(&self) -> u64 {
        self.hist.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.hist.lock().unwrap().is_empty()
    }

    pub fn buckets(&self) -> u8 {
        self.hist.lock().unwrap().buckets()
    }

    pub fn is_auto_resize(&self) -> bool {
        self.hist.lock().unwrap().is_auto_resize()
    }

    pub fn clone_correct(&self, interval: u64) -> PyResult<HdrHistogram> {
        let h = HdrHistogram {
            hist: Arc::new(Mutex::new(self.hist.lock().unwrap().clone_correct(interval),))
        };
        Ok(h)
    }

    pub fn set_to(&self, source:&Self) -> PyResult<()> {
        let _ = self.hist.lock().unwrap().set_to(source.hist.lock().unwrap().deref_mut());
        Ok(())
    }

    pub fn set_to_corrected(&self, source:&Self, interval:u64) -> PyResult<()> {
        let _ = self.hist.lock().unwrap().set_to_corrected(source.hist.lock().unwrap().deref_mut(), interval);
        Ok(())
    }

    pub fn add(&self, source:&Self) -> PyResult<()> {
        let _ = self.hist.lock().unwrap().add(source.hist.lock().unwrap().deref_mut());
        Ok(())
    }

    pub fn add_correct(&self, source:&Self, interval:u64) -> PyResult<()> {
        let _ = self.hist.lock().unwrap().add_correct(source.hist.lock().unwrap().deref_mut(), interval);
        Ok(())
    }

    pub fn subtract(&self, subtrahend:&Self) -> PyResult<()> {
        let _ = self.hist.lock().unwrap().subtract(subtrahend.hist.lock().unwrap().deref_mut());
        Ok(())
    }



    pub fn reset(&self) {
        self.hist.lock().unwrap().reset()
    }

    pub fn auto(&self, enabled:bool) {
        self.hist.lock().unwrap().auto(enabled)
    }

    #[new]
    pub fn new(sigfig:u8) -> PyResult<Self> {
        let h: Result<Histogram<u64>, CreationError> = Histogram::<u64>::new(sigfig);
        if let Ok(h) = h {
            let h = HdrHistogram {
                hist: Arc::new(Mutex::new(h))
            };
            Ok(h)
        } else {
            Err(PyValueError::new_err("Error in new"))
        }
    }

    #[staticmethod]
    pub fn new_with_max(high: u64, sigfig:u8) -> PyResult<Self> {
        let h: Result<Histogram<u64>, CreationError> = Histogram::<u64>::new_with_max(high, sigfig);
        if let Ok(h) = h {
            let h = HdrHistogram {
                hist: Arc::new(Mutex::new(h))
            };
            Ok(h)
        } else {
            Err(PyValueError::new_err("Error in new_with_max"))
        }
    }


    #[staticmethod]
    pub fn new_with_bounds(low:u64, high: u64, sigfig:u8) -> PyResult<Self> {
        let h: Result<Histogram<u64>, CreationError> = Histogram::<u64>::new_with_bounds(low, high, sigfig);
        if let Ok(h) = h {
            let h = HdrHistogram {
                hist: Arc::new(Mutex::new(h))
            };
            Ok(h)
        } else {
            Err(PyValueError::new_err("Error in new_with_bounds"))
        }
    }

    #[staticmethod]
    pub fn new_from(source: &HdrHistogram) -> PyResult<Self> {
        let h= Histogram::<u64>::new_from(&source.hist.lock().unwrap());

            let h = HdrHistogram {
                hist: Arc::new(Mutex::new(h))
            };
            Ok(h)

    }

    pub fn record(&self, value: u64) -> PyResult<()> {
        self.hist.lock().unwrap().record(value)
            .map_err(|e| PyValueError::new_err(format!("Failed to record value: {}", e)))
    }

    pub fn saturating_record(&self, value: u64) -> () {
        self.hist.lock().unwrap().saturating_record(value)
    }

    pub fn record_n(&self, value: u64, count: u64) -> PyResult<()> {
        self.hist.lock().unwrap().record_n(value, count)
            .map_err(|e| PyValueError::new_err(format!("Failed to record_n value: {}", e)))
    }

    pub fn saturating_record_n(&self, value: u64, count: u64) -> () {
        self.hist.lock().unwrap().saturating_record_n(value, count)
    }

    pub fn record_correct(&self, value: u64, interval: u64) -> PyResult<()> {
        self.hist.lock().unwrap().record_correct(value, interval)
            .map_err(|e| PyValueError::new_err(format!("Failed to record_correct value: {}", e)))
    }

    pub fn record_n_correct(&self, value: u64, count: u64, interval: u64) -> PyResult<()> {
        self.hist.lock().unwrap().record_n_correct(value, count, interval)
            .map_err(|e| PyValueError::new_err(format!("Failed to record_n_correct value: {}", e)))
    }

    pub fn min(&self) -> u64 {
        self.hist.lock().unwrap().min()
    }

    pub fn max(&self) -> u64 {
        self.hist.lock().unwrap().max()
    }

    pub fn min_nz(&self) -> u64 {
        self.hist.lock().unwrap().min_nz()
    }

    pub fn equivalent(&self, value: u64, other: u64) -> bool {
        self.hist.lock().unwrap().equivalent(value, other)
    }

    pub fn mean(&self) -> f64 {
        self.hist.lock().unwrap().mean()
    }

    pub fn stdev(&self) -> f64 {
        self.hist.lock().unwrap().stdev()
    }

    pub fn value_at_percentile(&self, percentile: f64) -> u64 {
        self.hist.lock().unwrap().value_at_percentile(percentile)
    }

    pub fn value_at_quantile(&self, quantile: f64) -> u64 {
        self.hist.lock().unwrap().value_at_quantile(quantile)
    }

    pub fn percentile_below(&self, value: u64) -> f64 {
        self.hist.lock().unwrap().percentile_below(value)
    }

    pub fn quantile_below(&self, value: u64) -> f64 {
        self.hist.lock().unwrap().quantile_below(value)
    }

    pub fn count_between(&self, low: u64, high: u64) -> u64 {
        self.hist.lock().unwrap().count_between(low, high)
    }

    pub fn count_at(&self, value: u64) -> u64 {
        self.hist.lock().unwrap().count_at(value)
    }

    pub fn lowest_equivalent(&self, value: u64) -> u64 {
        self.hist.lock().unwrap().lowest_equivalent(value)
    }

    pub fn highest_equivalent(&self, value: u64) -> u64 {
        self.hist.lock().unwrap().highest_equivalent(value)
    }

    pub fn next_non_equivalent(&self, value: u64) -> u64 {
        self.hist.lock().unwrap().next_non_equivalent(value)
    }

    pub fn equivalent_range(&self, value: u64) -> u64 {
        self.hist.lock().unwrap().equivalent_range(value)
    }

    pub fn iter_quantiles(&self, ticks_per_half_distance: u32) -> Vec<PyIterationValue> {
        self.hist.lock().unwrap().iter_quantiles(ticks_per_half_distance)
            .map(|iv| PyIterationValue(iv)).collect()
    }

    pub fn iter_linear(&self, step: u64) -> Vec<PyIterationValue> {
        self.hist.lock().unwrap().iter_linear(step)
            .map(|iv| PyIterationValue(iv)).collect()
    }

    pub fn iter_log(&self, start: u64, exp:f64) -> Vec<PyIterationValue> {
        self.hist.lock().unwrap().iter_log(start, exp)
            .map(|iv| PyIterationValue(iv)).collect()
    }


    pub fn iter_recorded(&self) -> Vec<PyIterationValue> {
        self.hist.lock().unwrap().iter_recorded()
            .map(|iv| PyIterationValue(iv)).collect()
    }


    pub fn iter_all(&self) -> Vec<PyIterationValue> {
        self.hist.lock().unwrap().iter_all()
            .map(|iv| PyIterationValue(iv)).collect()
    }
}


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

#[pymethods]
impl HdrIntervalLogHistogram {
    pub fn tag(&self) -> Option<String> {
        self.tag.clone()
    }

    pub fn start_timestamp(&self) -> DateTime<Utc> {
        self.start_timestamp
    }


    pub fn duration(&self) -> time::Duration {
        self.duration
    }

    pub fn max(&self) -> f64 {
        self.max
    }


    pub fn decoded_histogram(&self) -> Vec<u8> {
        self.decoded_histogram.clone()
    }

    pub fn hist(&self) -> PyResult<HdrHistogram> {
         Ok(HdrHistogram {
            hist: Arc::new(Mutex::new(from_decoded_histogram(self.decoded_histogram()).unwrap())),
        })


    }


}
#[pymethods]
impl HistogramLogReader {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let buf = load_interval_log_from_file(Path::new(path));
        Ok(HistogramLogReader {
            hists: buf.into_iter().filter_map(|q| match q {
                Ok(LogEntry::Interval(interval)) => Some(HdrIntervalLogHistogram{
                    tag: Some(String::from(interval.tag().unwrap().as_str())),
                    start_timestamp: from_duration(interval.start_timestamp()),
                    duration: interval.duration(),
                    max: interval.max(),
                    decoded_histogram: BASE64_STANDARD.decode(interval.encoded_histogram()).unwrap(),

                }), _ => None,
            })
                .collect(),
        })
    }

    fn hists(&self) -> Vec<HdrIntervalLogHistogram> {
        self.hists.clone()
    }
}

#[pymethods]
impl HistogramLogWriter {}