from hdrhistogram_pyo3._internal import HdrHistogram, HdrIntervalLogHistogram, HistogramLogReader, HistogramLogWriter
h = HistogramLogReader("SFA_001.perf")
print(dir(h))
for hdr in h.hists():
    hist = hdr.hist()
    print(f"Mean: {hist.mean()}, P50: {hist.value_at_percentile(50)}, P95: {hist.value_at_percentile(95)}, P99: {hist.value_at_percentile(99)}")
    for iq in hist.iter_quantiles(10):
        print(f"Quantile: {iq.quantile()}/{iq.quantile_iterated_to()}, Value: {iq.value_iterated_to()}")