mod expressions;
use pyo3::prelude::*;
use pyo3_polars::PolarsAllocator;
use crate::expressions::{HdrHistogram, HdrIntervalLogHistogram, HistogramLogReader, HistogramLogWriter};

#[pymodule]
fn _internal(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<HdrHistogram>()?;
    m.add_class::<HdrIntervalLogHistogram>()?;
    m.add_class::<HistogramLogReader>()?;
    m.add_class::<HistogramLogWriter>()?;
    Ok(())
}

#[global_allocator]
static ALLOC: PolarsAllocator = PolarsAllocator::new();

extern crate hdrhistogram;